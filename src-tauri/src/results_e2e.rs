//! End to end: a scripted race written as LMU shared memory, frame by frame,
//! through the same steps as the telemetry loop (parse, recorder, results
//! tracker) into the database.

use std::sync::Arc;

use crate::db::{Db, SessionSummary};
use crate::lmu::{
    self,
    layout::{self, ScoringInfo, TelemInfo, VehicleScoring},
};
use crate::model::Snapshot;
use crate::recorder::Recorder;
use crate::results::{FinishStatus, ResultEntry, ResultsTracker};

const LEN: f64 = 4000.0;
const RACE_LAPS: i32 = 5;
/// The countdown runs until this session time, then the race goes green.
const GREEN_ET: f64 = 10.0;
const DT: f64 = 0.25;

fn put<T: Copy>(buf: &mut [u8], offset: usize, value: T) {
    assert!(offset + std::mem::size_of::<T>() <= buf.len());
    // SAFETY: bounds checked above; the layout types are plain old data.
    unsafe { std::ptr::write_unaligned(buf.as_mut_ptr().add(offset) as *mut T, value) }
}

fn zeroed<T: Copy>() -> T {
    // SAFETY: the layout types are plain old data, valid when all zero.
    unsafe { std::mem::zeroed() }
}

fn text<const N: usize>(s: &str) -> [u8; N] {
    let mut out = [0u8; N];
    out[..s.len()].copy_from_slice(s.as_bytes());
    out
}

struct Car {
    id: i32,
    driver: &'static str,
    entry: &'static str,
    model: &'static str,
    class: &'static str,
    /// m/s
    pace: f64,
    player: bool,
    retire_at: Option<f64>,
    // Simulation state.
    progress: f64,
    lap_start: f64,
    best: f64,
    pits: i16,
    status: i8,
    finished_at: f64,
}

impl Car {
    fn laps(&self) -> i32 {
        (self.progress / LEN).floor().max(0.0) as i32
    }
}

#[allow(clippy::too_many_arguments)]
fn car(id: i32, driver: &'static str, entry: &'static str, model: &'static str, class: &'static str, pace: f64, slot: f64) -> Car {
    Car {
        id,
        driver,
        entry,
        model,
        class,
        pace,
        player: false,
        retire_at: None,
        progress: -slot * 20.0,
        lap_start: GREEN_ET,
        best: 0.0,
        pits: 0,
        status: 0,
        finished_at: f64::INFINITY,
    }
}

struct Race {
    cars: Vec<Car>,
    et: f64,
    phase: u8,
}

impl Race {
    fn new() -> Self {
        let mut player = car(3, "Me Myself", "AF Corse 2026 #83:LM", "Ferrari 499P", "Hypercar", 41.0, 4.0);
        player.player = true;
        let mut retiring = car(6, "G. Three", "TF Sport 2026 #33:LM", "Corvette Z06 GT3.R", "GT3", 35.5, 5.0);
        retiring.retire_at = Some(GREEN_ET + 250.0);
        Self {
            cars: vec![
                car(1, "A. Leader", "Toyota Gazoo Racing 2026 #7:LM", "Toyota TR010 Hybrid", "Hypercar", 41.5, 0.0),
                // Starts ahead of the player but is slower: the player passes it.
                car(2, "B. Second", "Porsche Penske 2026 #6:LM", "Porsche 963", "Hypercar", 40.6, 1.0),
                player,
                car(4, "C. GT", "Manthey 2026 #92:LM", "Porsche 911 GT3 R", "GT3", 36.0, 2.0),
                car(5, "D. GT", "Vista AF Corse 2026 #21:LM", "Ferrari 296 GT3", "GT3", 35.8, 3.0),
                retiring,
            ],
            et: 0.0,
            phase: 4,
        }
    }

    fn step(&mut self) {
        self.et += DT;
        if self.et < GREEN_ET {
            return;
        }
        self.phase = 5;
        let leader_done = self.cars.iter().any(|c| c.status == 1);
        for c in &mut self.cars {
            if c.status != 0 {
                continue;
            }
            if c.retire_at.is_some_and(|t| self.et >= t) {
                c.status = 2;
                continue;
            }
            let before = c.laps();
            c.progress += c.pace * DT;
            if c.laps() > before && c.progress > 0.0 {
                if before > 0 {
                    let lap = self.et - c.lap_start;
                    if c.best == 0.0 || lap < c.best {
                        c.best = lap;
                    }
                }
                c.lap_start = self.et;
                if c.id == 4 && c.laps() == 2 {
                    c.pits += 1;
                }
                // The leader takes the flag after the race distance, everyone else on their next crossing.
                if c.laps() >= RACE_LAPS || leader_done {
                    c.status = 1;
                    c.finished_at = self.et;
                }
            }
        }
        if self.cars.iter().all(|c| c.status != 0) {
            self.phase = 8;
        }
    }

    /// Cars in race order: distance, then who crossed the line first.
    fn order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.cars.len()).collect();
        let key = |c: &Car| (c.laps(), if c.status == 1 { 1e6 - c.finished_at } else { c.progress - c.laps() as f64 * LEN });
        order.sort_by(|&a, &b| key(&self.cars[b]).partial_cmp(&key(&self.cars[a])).unwrap());
        order
    }

    /// The `LMU_Data` block the game would publish for this frame.
    fn shared_memory(&self) -> Vec<u8> {
        let mut buf = vec![0u8; layout::TOTAL_SIZE];
        let mut info: ScoringInfo = zeroed();
        info.track_name = text("Test Ring");
        info.session = 10;
        info.current_et = self.et;
        info.max_laps = RACE_LAPS;
        info.lap_dist = LEN;
        info.num_vehicles = self.cars.len() as i32;
        info.game_phase = self.phase;
        info.in_realtime = 1;
        info.player_name = text("Me Myself");
        put(&mut buf, layout::OFF_SCORING_INFO, info);

        let order = self.order();
        let leader = &self.cars[order[0]];
        for (place, &i) in order.iter().enumerate() {
            let c = &self.cars[i];
            let mut v: VehicleScoring = zeroed();
            v.id = c.id;
            v.driver_name = text(c.driver);
            v.vehicle_name = text(c.entry);
            v.vehicle_class = text(c.class);
            v.pit_group = text(c.entry.split(" 2026").next().unwrap());
            v.veh_filename = text(&format!("{}_26_TEST.VEH", c.id));
            v.total_laps = c.laps() as i16;
            v.lap_dist = c.progress.rem_euclid(LEN);
            v.finish_status = c.status;
            v.best_lap_time = if c.best > 0.0 { c.best } else { -1.0 };
            v.num_pitstops = c.pits;
            v.is_player = c.player as u8;
            v.place = place as u8 + 1;
            // Worst case for finished cars: the live gap collapses once both are stopped at the line.
            let behind = leader.progress - c.progress;
            v.time_behind_leader = behind / c.pace;
            v.laps_behind_leader = (behind / LEN).floor() as i32;
            v.lap_start_et = c.lap_start;
            put(&mut buf, layout::OFF_SCORING_VEHICLES + i * std::mem::size_of::<VehicleScoring>(), v);
        }

        put(&mut buf, layout::OFF_TELEMETRY_ACTIVE_VEHICLES, self.cars.len() as u8);
        put(&mut buf, layout::OFF_TELEMETRY_PLAYER_HAS_VEHICLE, 1u8);
        for (i, c) in self.cars.iter().enumerate() {
            let mut t: TelemInfo = zeroed();
            t.id = c.id;
            t.vehicle_name = text(c.entry);
            t.vehicle_model = text(c.model);
            t.elapsed_time = self.et;
            t.lap_number = c.laps();
            t.lap_start_et = c.lap_start;
            t.current_sector = 1;
            put(&mut buf, layout::OFF_TELEMETRY_INFO + i * std::mem::size_of::<TelemInfo>(), t);
            if c.player {
                put(&mut buf, layout::OFF_TELEMETRY_PLAYER_IDX, i as u8);
            }
        }
        buf
    }
}

struct App {
    db: Arc<Db>,
    recorder: Recorder,
    results: ResultsTracker,
    saves: Vec<Vec<ResultEntry>>,
}

impl App {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("lmo-race-e2e-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("r.db");
        let _ = std::fs::remove_file(&path);
        let db = Arc::new(Db::open(&path).unwrap());
        let (tx, _rx) = std::sync::mpsc::channel();
        Self { recorder: Recorder::new(db.clone(), tx), db, results: ResultsTracker::default(), saves: Vec::new() }
    }

    /// One pass of the telemetry loop.
    fn frame(&mut self, mut snap: Snapshot) {
        self.recorder.update(&mut snap);
        if let Some(r) = self.results.update(&snap, self.recorder.session()) {
            self.saves.push(r.entries.clone());
            self.db.save_results(&r).unwrap();
        }
    }

    fn race_session(&self) -> SessionSummary {
        let sessions = self.db.list_sessions(None, None, 10, 0).unwrap();
        assert_eq!(sessions.len(), 1);
        sessions.into_iter().next().unwrap()
    }
}

fn player_position(entries: &[ResultEntry]) -> i32 {
    entries.iter().find(|e| e.is_player).unwrap().position
}

#[test]
fn a_finished_race_stores_the_full_classification() {
    let mut race = Race::new();
    let mut app = App::new("finish");
    while race.phase != 8 {
        race.step();
        app.frame(lmu::parse(&race.shared_memory()));
        assert!(race.et < 2000.0, "race never ended");
    }
    for _ in 0..20 {
        race.step();
        app.frame(lmu::parse(&race.shared_memory()));
    }
    // The game closes.
    app.frame(Snapshot::default());

    let session = app.race_session();
    assert_eq!(session.session.session_type, "Race");
    assert_eq!(session.results.as_deref(), Some("complete"));
    let rows = app.db.session_results(session.session.id).unwrap();
    assert_eq!(rows, *app.saves.last().unwrap());
    assert_eq!(rows.len(), 6);

    let expected: Vec<i32> = race.order().iter().map(|&i| race.cars[i].id).collect();
    // Entries carry no id, so match on the driver instead.
    let by_driver: Vec<i32> = rows.iter().map(|r| race.cars.iter().find(|c| c.driver == r.driver).unwrap().id).collect();
    assert_eq!(by_driver, expected);
    assert_eq!(rows.iter().map(|r| r.position).collect::<Vec<_>>(), vec![1, 2, 3, 4, 5, 6]);

    let row = |driver: &str| rows.iter().find(|r| r.driver == driver).unwrap();
    let leader = row("A. Leader");
    assert_eq!((leader.position, leader.laps, leader.status), (1, RACE_LAPS, FinishStatus::Finished));
    assert_eq!(leader.number, "7");
    assert_eq!(leader.car, "Toyota TR010 Hybrid");
    assert_eq!(leader.team, "Toyota Gazoo Racing");
    assert_eq!(leader.veh_file, "1_26_TEST");
    assert!(leader.best_lap.unwrap() > 90.0 && leader.best_lap.unwrap() < 100.0);

    // The player passed the slower Porsche and finished second.
    let me = row("Me Myself");
    assert!(me.is_player);
    assert_eq!((me.position, me.class_position, me.status), (2, 2, FinishStatus::Finished));
    assert!(app.saves.iter().any(|s| player_position(s) > 2), "the pass was never saved");
    assert_eq!((session.position, session.class_position), (Some(2), Some(2)));

    // GT3s are classified within their class; the retired car is last with a DNF.
    let gt = row("C. GT");
    assert_eq!((gt.class_position, gt.car_class.as_str(), gt.pit_stops), (1, "GT3", 1));
    let out = row("G. Three");
    assert_eq!((out.position, out.class_position, out.status), (6, 3, FinishStatus::Dnf));
    assert!(out.laps < gt.laps);
    // Gaps at the flag come from when each car crossed the line.
    let winner_et = race.cars.iter().find(|c| c.driver == "A. Leader").unwrap().finished_at;
    for r in rows.iter().filter(|r| r.status == FinishStatus::Finished && r.laps == leader.laps && r.position > 1) {
        let crossed = race.cars.iter().find(|c| c.driver == r.driver).unwrap().finished_at;
        assert!(r.gap > 0.0 && (r.gap - (crossed - winner_et)).abs() < DT * 1.01, "{} gap {}", r.driver, r.gap);
    }
    // Race times run from the green flag to each car's own finish.
    for r in &rows {
        let c = race.cars.iter().find(|c| c.driver == r.driver).unwrap();
        match r.status {
            FinishStatus::Finished => {
                let t = r.race_time.unwrap_or_else(|| panic!("{} has no race time", r.driver));
                assert!((t - (c.finished_at - GREEN_ET)).abs() < DT * 1.01, "{} race time {t}", r.driver);
            }
            _ => assert_eq!(r.race_time, None, "{}", r.driver),
        }
    }
    let gaps: Vec<f64> = rows.iter().filter(|r| r.laps == leader.laps).map(|r| r.gap).collect();
    assert!(gaps.windows(2).all(|w| w[0] <= w[1]), "gaps out of order: {gaps:?}");

    // Saved often enough to follow the race, but not every frame.
    let frames = (race.et / DT) as usize;
    assert!(app.saves.len() > 5 && app.saves.len() < frames / 20, "{} saves over {frames} frames", app.saves.len());
}

#[test]
fn quitting_mid_race_keeps_a_partial_result() {
    let mut race = Race::new();
    let mut app = App::new("quit");
    // Countdown only: nothing stored yet.
    while race.phase != 5 {
        race.step();
        app.frame(lmu::parse(&race.shared_memory()));
    }
    assert!(app.db.list_sessions(None, None, 10, 0).unwrap().is_empty());

    while race.cars.iter().map(|c| c.laps()).max().unwrap() < 3 {
        race.step();
        app.frame(lmu::parse(&race.shared_memory()));
    }
    // The player leaves for the main menu: the game stops reporting a session.
    app.frame(lmu::parse(&vec![0u8; layout::TOTAL_SIZE]));

    let session = app.race_session();
    assert_eq!(session.results.as_deref(), Some("partial"));
    let rows = app.db.session_results(session.session.id).unwrap();
    assert_eq!(rows.len(), 6);
    // Only the car that retired early has a final status.
    for r in &rows {
        let expected = if r.driver == "G. Three" { FinishStatus::Dnf } else { FinishStatus::Running };
        assert_eq!(r.status, expected, "{}", r.driver);
    }
    // The last frame before quitting, not an older save.
    let expected: Vec<&str> = race.order().iter().map(|&i| race.cars[i].driver).collect();
    assert_eq!(rows.iter().map(|r| r.driver.as_str()).collect::<Vec<_>>(), expected);
    assert_eq!(rows[0].laps, 3);
}
