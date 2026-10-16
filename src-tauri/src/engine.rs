//! Background threads: the data loop (game or demo -> recorder -> UI events)
//! and the database writer for finished laps.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::db::{self, Db, NewLap};
use crate::demo::DemoSim;
use crate::lmu::{self, layout, shm::{self, LmuSharedMemory}};
use crate::model::Snapshot;
use crate::recorder::Recorder;

pub struct Shared {
    pub latest: RwLock<Snapshot>,
    pub damage: RwLock<Option<crate::damage::Reading>>,
    pub demo: AtomicBool,
    /// Max snapshot events per second sent to the windows.
    pub emit_hz: AtomicU32,
    pub reload_reference: AtomicBool,
    /// HWND of the running game window, 0 when not connected.
    pub game_window: AtomicU64,
    /// Whether the game window currently has keyboard focus.
    pub game_focused: AtomicBool,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            latest: RwLock::new(Snapshot::default()),
            damage: RwLock::new(None),
            demo: AtomicBool::new(false),
            emit_hz: AtomicU32::new(30),
            reload_reference: AtomicBool::new(false),
            game_window: AtomicU64::new(0),
            game_focused: AtomicBool::new(false),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LapRecorded {
    lap_id: i64,
    session_id: i64,
    lap_number: i32,
    lap_time: f64,
    valid: bool,
    personal_best: bool,
    track: String,
}

pub fn spawn(app: AppHandle, shared: Arc<Shared>, db: Arc<Db>) {
    crate::damage::spawn(shared.clone());
    let (tx, rx) = mpsc::channel::<NewLap>();
    {
        let app = app.clone();
        let db = db.clone();
        thread::Builder::new()
            .name("lap-writer".into())
            .spawn(move || run_writer(app, db, rx))
            .expect("spawn lap writer");
    }
    {
        let app = app.clone();
        let shared = shared.clone();
        thread::Builder::new()
            .name("focus-watch".into())
            .spawn(move || run_focus_watch(app, shared))
            .expect("spawn focus watcher");
    }
    thread::Builder::new()
        .name("telemetry".into())
        .spawn(move || run_loop(app, shared, Recorder::new(db, tx)))
        .expect("spawn telemetry loop");
}

fn run_writer(app: AppHandle, db: Arc<Db>, rx: Receiver<NewLap>) {
    for lap in rx {
        let s = &lap.session;
        let previous_best = db.best_lap(&s.track, &s.car).ok().flatten();
        let id = match db.insert_lap(&lap) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("failed to store lap: {e}");
                continue;
            }
        };
        let clean = lap.valid && !lap.out_lap && !lap.in_lap;
        let personal_best = clean && previous_best.is_none_or(|(_, t, _)| lap.lap_time < t);
        let has_map = db.track_map(&s.track).ok().flatten().is_some();
        if clean && (personal_best || !has_map) && s.track_length > 0.0 {
            let map = db::track_map_from_trace(&s.track, s.track_length, &lap.trace, 5.0);
            if db.save_track_map(&map, id).is_ok() {
                let _ = app.emit("track-map-updated", &s.track);
            }
        }
        let _ = app.emit(
            "lap-recorded",
            LapRecorded {
                lap_id: id,
                session_id: s.id,
                lap_number: lap.lap_number,
                lap_time: lap.lap_time,
                valid: lap.valid,
                personal_best,
                track: s.track.clone(),
            },
        );
    }
}

/// Polls the foreground window and emits `game-focused` when the game gains or
/// loses focus, so the overlay can hide while alt-tabbed.
fn run_focus_watch(app: AppHandle, shared: Arc<Shared>) {
    loop {
        let focused = shm::is_foreground(shared.game_window.load(Ordering::Acquire));
        if shared.game_focused.swap(focused, Ordering::AcqRel) != focused {
            let _ = app.emit("game-focused", focused);
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn run_loop(app: AppHandle, shared: Arc<Shared>, mut recorder: Recorder) {
    let mut buf = vec![0u8; layout::TOTAL_SIZE];
    let mut shm: Option<LmuSharedMemory> = None;
    let mut next_open = Instant::now();
    let mut demo: Option<DemoSim> = None;
    let mut last_tick = Instant::now();
    let mut last_emit = Instant::now() - Duration::from_secs(1);
    let mut last_change = Instant::now();
    let mut last_key = (f64::NAN, f64::NAN);
    let mut frame = 0u64;

    loop {
        if shared.reload_reference.swap(false, Ordering::AcqRel) {
            recorder.invalidate_reference();
        }

        if shm.is_none() {
            shared.game_window.store(0, Ordering::Release);
        }

        let mut snap = if shared.demo.load(Ordering::Acquire) {
            shm = None;
            thread::sleep(Duration::from_millis(16));
            let now = Instant::now();
            let dt = (now - last_tick).as_secs_f64().min(0.1);
            last_tick = now;
            demo.get_or_insert_with(DemoSim::new).step(dt)
        } else {
            demo = None;
            if shm.is_none() && Instant::now() >= next_open {
                shm = LmuSharedMemory::open();
                next_open = Instant::now() + Duration::from_secs(2);
                last_change = Instant::now();
            }
            match &shm {
                None => {
                    thread::sleep(Duration::from_millis(250));
                    Snapshot::default()
                }
                Some(mem) => {
                    let got_frame = mem.wait_for_frame(250);
                    if !mem.copy_into(&mut buf) {
                        continue;
                    }
                    if !mem.game_alive(&buf) {
                        shm = None;
                        Snapshot::default()
                    } else {
                        shared.game_window.store(shm::game_window(&buf), Ordering::Release);
                        let s = lmu::parse(&buf);
                        let key = (s.session.current_et, s.player.as_ref().map_or(0.0, |p| p.elapsed_time));
                        if key != last_key {
                            last_key = key;
                            last_change = Instant::now();
                        } else if !got_frame && last_change.elapsed() > Duration::from_secs(20) {
                            // No updates for a long time: re-open to detect a closed game.
                            shm = None;
                        }
                        s
                    }
                }
            }
        };

        frame += 1;
        if let Ok(reading) = shared.damage.read() {
            if let Some(reading) = reading.as_ref() {
                reading.apply(&mut snap);
            }
        }
        snap.frame = frame;
        recorder.update(&mut snap);

        let hz = shared.emit_hz.load(Ordering::Relaxed).clamp(1, 120);
        if last_emit.elapsed() >= Duration::from_secs_f64(1.0 / hz as f64) {
            last_emit = Instant::now();
            let _ = app.emit("snapshot", &snap);
            if let Ok(mut latest) = shared.latest.write() {
                *latest = snap;
            }
        }
    }
}

/// Demo data for the widget previews in the companion app. Runs its own
/// simulation, separate from demo mode, so it never shows up on the overlay
/// and finished laps are discarded instead of being written to the database.
pub struct PreviewSim {
    sim: DemoSim,
    recorder: Recorder,
    last: Instant,
    frame: u64,
}

impl PreviewSim {
    /// Simulated time run up front so lap times, deltas and fuel averages are
    /// already filled in when the preview first appears.
    const WARMUP_SECS: f64 = 300.0;
    const STEP_SECS: f64 = 0.02;

    pub fn new(db: Arc<Db>) -> Self {
        // The receiver is dropped, so laps the recorder finishes go nowhere.
        let (tx, _) = mpsc::channel::<NewLap>();
        let mut preview = Self { sim: DemoSim::new(), recorder: Recorder::new(db, tx), last: Instant::now(), frame: 0 };
        preview.advance(Self::WARMUP_SECS);
        preview
    }

    /// Advances the simulation by the wall time since the previous call
    /// (capped, so it resumes smoothly after the preview was hidden).
    pub fn next(&mut self) -> Snapshot {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f64().min(0.1);
        self.last = now;
        self.advance(dt)
    }

    fn advance(&mut self, secs: f64) -> Snapshot {
        let steps = (secs / Self::STEP_SECS).ceil().max(1.0) as usize;
        let dt = secs / steps as f64;
        let mut snap = Snapshot::default();
        for _ in 0..steps {
            snap = self.sim.step(dt);
            self.frame += 1;
            snap.frame = self.frame;
            self.recorder.update(&mut snap);
        }
        snap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_is_warmed_up_and_not_recorded() {
        let dir = std::env::temp_dir().join(format!("lmo-preview-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("p.db");
        let _ = std::fs::remove_file(&path);
        let db = Arc::new(Db::open(&path).unwrap());

        let mut preview = PreviewSim::new(db.clone());
        let snap = preview.next();

        assert!(snap.player.is_some());
        assert!(snap.timing.last_lap_time > 0.0);
        assert!(snap.strategy.fuel_per_lap > 0.0);
        assert!(db.list_sessions(None, None, 10, 0).unwrap().is_empty());
    }
}
