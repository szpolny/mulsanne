//! Turns the snapshot stream into recorded laps, live timing and strategy.

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::{Db, LapTrace, NewLap, SessionRecord};
use crate::model::{Player, Snapshot, Source};

/// Telemetry is stored at this rate.
const SAMPLE_INTERVAL: f64 = 1.0 / 30.0;
/// Number of recent green-flag laps used for fuel/VE averages.
const STRATEGY_WINDOW: usize = 5;

fn unix_millis() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn avg(v: &VecDeque<f64>) -> f64 {
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

pub fn car_key(p: &Player) -> String {
    if p.vehicle_model.is_empty() {
        p.vehicle_name.clone()
    } else {
        p.vehicle_model.clone()
    }
}

/// Reference lap used for the live delta: time as a function of distance.
struct Reference {
    d: Vec<f32>,
    t: Vec<f32>,
}

impl Reference {
    fn from_trace(trace: &LapTrace) -> Option<Self> {
        if trace.len() < 10 {
            return None;
        }
        // Keep distance strictly increasing so binary search is well-defined.
        let mut d = Vec::with_capacity(trace.len());
        let mut t = Vec::with_capacity(trace.len());
        for i in 0..trace.len() {
            if d.last().is_none_or(|&last| trace.d[i] > last) {
                d.push(trace.d[i]);
                t.push(trace.t[i]);
            }
        }
        Some(Self { d, t })
    }

    fn time_at(&self, dist: f64) -> Option<f64> {
        let dist = dist as f32;
        if dist < self.d[0] || dist > *self.d.last()? {
            return None;
        }
        let i = self.d.partition_point(|&x| x < dist).max(1);
        let (d0, d1) = (self.d[i - 1], self.d[i]);
        let (t0, t1) = (self.t[i - 1], self.t[i]);
        let f = if d1 > d0 { (dist - d0) / (d1 - d0) } else { 0.0 };
        Some((t0 + (t1 - t0) * f) as f64)
    }
}

struct LapState {
    lap_number: i32,
    lap_start_et: f64,
    from_start: bool,
    out_lap: bool,
    entered_pits: bool,
    invalid: bool,
    yellow: bool,
    start_fuel: f64,
    start_ve: f64,
    splits: [Option<f64>; 2],
    prev_sector: i8,
    dist: f64,
    last_scoring_dist: f64,
    last_sample_t: f64,
    trace: LapTrace,
}

impl LapState {
    fn new(p: &Player) -> Self {
        let t = p.elapsed_time - p.lap_start_et;
        Self {
            lap_number: p.lap_number,
            lap_start_et: p.lap_start_et,
            // We only keep laps observed from the start line.
            from_start: t < 1.0,
            out_lap: p.in_pits,
            entered_pits: false,
            invalid: p.lap_invalidated,
            yellow: false,
            start_fuel: p.fuel,
            start_ve: p.virtual_energy,
            splits: [None, None],
            prev_sector: p.sector,
            dist: if t < 1.0 { 0.0 } else { p.lap_dist },
            last_scoring_dist: p.lap_dist,
            last_sample_t: f64::NEG_INFINITY,
            trace: LapTrace::default(),
        }
    }

    fn time(&self, p: &Player) -> f64 {
        p.elapsed_time - self.lap_start_et
    }

    fn update(&mut self, p: &Player, dt: f64, track_length: f64, phase: u8) {
        let t = self.time(p);

        // Scoring distance updates less often than telemetry; integrate speed
        // in between and snap whenever scoring moves.
        let mut next = self.dist + p.speed * dt;
        if p.lap_dist != self.last_scoring_dist {
            self.last_scoring_dist = p.lap_dist;
            let fresh = p.lap_dist;
            // Right after the line scoring may still report the previous lap.
            if !(t < 10.0 && fresh > track_length * 0.5) {
                next = fresh;
            }
        }
        if track_length > 0.0 {
            next = next.min(track_length);
        }
        self.dist = next.max(self.dist);

        self.invalid |= p.lap_invalidated;
        self.yellow |= phase == 6;
        if p.in_pits && !self.out_lap {
            self.entered_pits = true;
        }
        if p.sector != self.prev_sector {
            match (self.prev_sector, p.sector) {
                (1, 2) => self.splits[0] = Some(t),
                (2, 3) => self.splits[1] = Some(t),
                _ => {}
            }
            self.prev_sector = p.sector;
        }

        // Tolerance so float jitter on 60 Hz frames doesn't skip every other sample slot.
        if t - self.last_sample_t >= SAMPLE_INTERVAL * 0.75 {
            self.last_sample_t = t;
            let tr = &mut self.trace;
            tr.t.push(t as f32);
            tr.d.push(self.dist as f32);
            tr.speed.push(p.speed as f32);
            tr.throttle.push(p.throttle as f32);
            tr.brake.push(p.brake as f32);
            tr.steering.push(p.steering as f32);
            tr.clutch.push(p.clutch as f32);
            tr.gear.push(p.gear.clamp(-1, 20) as i8);
            tr.rpm.push(p.rpm as f32);
            tr.x.push(p.pos[0] as f32);
            tr.z.push(p.pos[2] as f32);
            tr.g_lat.push(p.g_lat as f32);
            tr.g_long.push(p.g_long as f32);
            tr.abs.push(p.abs_active as u8);
            tr.tc.push(p.tc_active as u8);
            let avg_temp = |i: usize| (p.wheels[i].surface_temp.iter().sum::<f64>() / 3.0) as f32;
            tr.tyre_fl.push(avg_temp(0));
            tr.tyre_fr.push(avg_temp(1));
            tr.tyre_rl.push(avg_temp(2));
            tr.tyre_rr.push(avg_temp(3));
        }
    }

    fn sectors(&self, lap_time: f64) -> [Option<f64>; 3] {
        match self.splits {
            [Some(a), Some(b)] if b > a && lap_time > b => [Some(a), Some(b - a), Some(lap_time - b)],
            [Some(a), _] => [Some(a), None, None],
            _ => [None, None, None],
        }
    }
}

struct SessionState {
    key: (Source, String, i32),
    record: SessionRecord,
    last_et: f64,
    best: f64,
    best_sectors: [f64; 3],
    last_lap_time: f64,
    last_lap_valid: bool,
    last_sectors: [f64; 3],
    stint_start_et: Option<f64>,
    stint_laps: u32,
    fuel_hist: VecDeque<f64>,
    ve_hist: VecDeque<f64>,
    time_hist: VecDeque<f64>,
}

pub struct Recorder {
    db: Arc<Db>,
    tx: Sender<NewLap>,
    session: Option<SessionState>,
    lap: Option<LapState>,
    last_elapsed: f64,
    reference_key: Option<(String, String)>,
    reference: Option<Reference>,
    pb_time: f64,
    pb_sectors: [f64; 3],
    /// Per car: (pit stops seen, laps completed when the current tyre set went on).
    tyre_sets: HashMap<i32, (i16, i16)>,
    tyre_track: String,
}

impl Recorder {
    pub fn new(db: Arc<Db>, tx: Sender<NewLap>) -> Self {
        Self {
            db,
            tx,
            session: None,
            lap: None,
            last_elapsed: f64::NEG_INFINITY,
            reference_key: None,
            reference: None,
            pb_time: 0.0,
            pb_sectors: [0.0; 3],
            tyre_sets: HashMap::new(),
            tyre_track: String::new(),
        }
    }

    /// Forces the personal best reference to be reloaded (e.g. after laps were deleted).
    pub fn invalidate_reference(&mut self) {
        self.reference_key = None;
    }

    fn load_reference(&mut self, track: &str, car: &str) {
        self.reference = None;
        self.pb_time = 0.0;
        self.pb_sectors = [0.0; 3];
        if let Ok(Some((id, time, sectors))) = self.db.best_lap(track, car) {
            self.pb_time = time;
            self.pb_sectors = sectors.map(|s| s.unwrap_or(0.0));
            if let Ok(trace) = self.db.lap_trace(id) {
                self.reference = Reference::from_trace(&trace);
            }
        }
        self.reference_key = Some((track.to_string(), car.to_string()));
    }

    /// Laps on the current tyres for every car, assuming each pit stop fits new tyres.
    fn track_tyre_sets(&mut self, snap: &mut Snapshot) {
        let key = format!("{}|{}", snap.session.track, snap.session.code);
        if key != self.tyre_track {
            self.tyre_track = key;
            self.tyre_sets.clear();
        }
        for v in &mut snap.vehicles {
            let entry = self.tyre_sets.entry(v.id).or_insert((v.num_pitstops, 0));
            if v.num_pitstops != entry.0 {
                *entry = (v.num_pitstops, v.total_laps);
            }
            v.tyre_laps = (v.total_laps - entry.1).max(0) as i32;
        }
    }

    pub fn update(&mut self, snap: &mut Snapshot) {
        self.track_tyre_sets(snap);
        let Some(p) = snap.player.clone() else {
            self.lap = None;
            return;
        };
        let track = snap.session.track.clone();
        let car = car_key(&p);

        // New session on track/session change or when the session clock restarts.
        let key = (snap.source, track.clone(), snap.session.code);
        let restarted = self.session.as_ref().is_some_and(|s| snap.session.current_et + 5.0 < s.last_et);
        if self.session.as_ref().is_none_or(|s| s.key != key) || restarted {
            let id = unix_millis();
            self.session = Some(SessionState {
                key,
                record: SessionRecord {
                    id,
                    started_at: id / 1000,
                    source: match snap.source {
                        Source::Demo => "demo",
                        _ => "lmu",
                    }
                    .into(),
                    track: track.clone(),
                    track_length: snap.session.track_length,
                    session_type: snap.session.kind.label().into(),
                    car: car.clone(),
                    vehicle_name: p.vehicle_name.clone(),
                    car_class: p.vehicle_class.clone(),
                    driver: snap.session.player_name.clone(),
                },
                last_et: snap.session.current_et,
                best: 0.0,
                best_sectors: [0.0; 3],
                last_lap_time: 0.0,
                last_lap_valid: true,
                last_sectors: [0.0; 3],
                stint_start_et: None,
                stint_laps: 0,
                fuel_hist: VecDeque::new(),
                ve_hist: VecDeque::new(),
                time_hist: VecDeque::new(),
            });
            self.lap = None;
            self.last_elapsed = f64::NEG_INFINITY;
        }
        if self.reference_key.as_ref() != Some(&(track.clone(), car.clone())) {
            self.load_reference(&track, &car);
        }

        let session = self.session.as_mut().expect("session set above");
        session.last_et = snap.session.current_et;
        // Car swaps within a session (e.g. test day) keep the session but
        // must be recorded against the right car.
        if session.record.car != car {
            session.record.car = car.clone();
            session.record.vehicle_name = p.vehicle_name.clone();
            session.record.car_class = p.vehicle_class.clone();
        }

        if p.in_pits {
            session.stint_start_et = None;
            session.stint_laps = 0;
        } else if session.stint_start_et.is_none() {
            session.stint_start_et = Some(p.elapsed_time);
        }

        let new_frame = p.elapsed_time > self.last_elapsed;
        let dt = if self.last_elapsed.is_finite() { (p.elapsed_time - self.last_elapsed).clamp(0.0, 0.5) } else { 0.0 };

        if !snap.in_realtime {
            // Sitting in the garage/monitor: lap in progress is not meaningful.
            self.lap = None;
        } else if new_frame {
            self.last_elapsed = p.elapsed_time;
            if let Some(lap) = &self.lap {
                if p.lap_number != lap.lap_number {
                    let done = self.lap.take().expect("checked above");
                    self.finish_lap(done, &p);
                }
            }
            let lap = self.lap.get_or_insert_with(|| LapState::new(&p));
            lap.update(&p, dt, snap.session.track_length, snap.session.phase);
        }

        self.fill_timing(snap, &p);
    }

    fn finish_lap(&mut self, lap: LapState, p: &Player) {
        let lap_time = p.lap_start_et - lap.lap_start_et;
        if !lap.from_start || lap_time < 5.0 || lap.trace.len() < 20 {
            return;
        }
        let session = self.session.as_mut().expect("lap implies session");
        let sectors = lap.sectors(lap_time);
        let valid = !lap.invalid;
        let in_lap = lap.entered_pits || p.in_pits;
        let clean = valid && !lap.out_lap && !in_lap;
        let fuel_used = (lap.start_fuel - p.fuel > 0.0).then_some(lap.start_fuel - p.fuel);
        let ve_used = (lap.start_ve - p.virtual_energy > 0.0).then_some(lap.start_ve - p.virtual_energy);

        session.last_lap_time = lap_time;
        session.last_lap_valid = valid;
        session.last_sectors = sectors.map(|s| s.unwrap_or(0.0));
        session.stint_laps += 1;
        if clean && (session.best == 0.0 || lap_time < session.best) {
            session.best = lap_time;
        }
        if valid {
            for (best, s) in session.best_sectors.iter_mut().zip(sectors) {
                if let Some(s) = s {
                    if *best == 0.0 || s < *best {
                        *best = s;
                    }
                }
            }
        }
        if !lap.out_lap && !in_lap && !lap.yellow {
            let push = |hist: &mut VecDeque<f64>, v: f64| {
                hist.push_back(v);
                if hist.len() > STRATEGY_WINDOW {
                    hist.pop_front();
                }
            };
            if let Some(f) = fuel_used {
                push(&mut session.fuel_hist, f);
            }
            if let Some(v) = ve_used {
                push(&mut session.ve_hist, v);
            }
            push(&mut session.time_hist, lap_time);
        }

        if clean && (self.pb_time == 0.0 || lap_time < self.pb_time) {
            self.pb_time = lap_time;
            self.pb_sectors = sectors.map(|s| s.unwrap_or(0.0));
            self.reference = Reference::from_trace(&lap.trace);
        }

        let _ = self.tx.send(NewLap {
            session: session.record.clone(),
            // LMU counts the lap in progress from 0; store the 1-based lap number.
            lap_number: lap.lap_number + 1,
            lap_time,
            sectors,
            valid,
            out_lap: lap.out_lap,
            in_lap,
            fuel_used,
            ve_used,
            recorded_at: unix_millis() / 1000,
            trace: lap.trace,
        });
    }

    fn fill_timing(&self, snap: &mut Snapshot, p: &Player) {
        let Some(session) = &self.session else { return };
        let scoring = snap.vehicles.iter().find(|v| v.is_player);
        let lap_time = p.elapsed_time - p.lap_start_et;
        let timing = &mut snap.timing;

        timing.current_lap_time = lap_time.max(0.0);
        timing.last_lap_time = if session.last_lap_time > 0.0 {
            session.last_lap_time
        } else {
            scoring.map(|v| v.last_lap.max(0.0)).unwrap_or(0.0)
        };
        timing.last_lap_valid = session.last_lap_valid;
        timing.session_best = if session.best > 0.0 {
            session.best
        } else {
            scoring.map(|v| v.best_lap.max(0.0)).unwrap_or(0.0)
        };
        timing.session_best_sectors = session.best_sectors;
        timing.last_sectors = session.last_sectors;
        timing.personal_best = self.pb_time;
        timing.personal_best_sectors = self.pb_sectors;
        timing.stint_laps = session.stint_laps;
        timing.stint_time = session.stint_start_et.map(|s| (p.elapsed_time - s).max(0.0)).unwrap_or(0.0);

        if let Some(lap) = &self.lap {
            let s1 = lap.splits[0];
            let s2 = lap.splits[1].zip(s1).map(|(b, a)| b - a);
            timing.current_sectors = [s1.unwrap_or(0.0), s2.unwrap_or(0.0), 0.0];
            if lap.from_start && lap.dist > 30.0 {
                if let Some(ref_t) = self.reference.as_ref().and_then(|r| r.time_at(lap.dist)) {
                    let delta = lap.time(p) - ref_t;
                    timing.delta_pb = Some(delta);
                    timing.predicted_lap = Some(self.pb_time + delta);
                }
            }
        }

        // Strategy
        let st = &mut snap.strategy;
        st.fuel_per_lap = avg(&session.fuel_hist);
        st.ve_per_lap = avg(&session.ve_hist);
        st.samples = session.time_hist.len() as u32;
        st.avg_lap_time = if !session.time_hist.is_empty() {
            avg(&session.time_hist)
        } else if timing.session_best > 0.0 {
            timing.session_best
        } else {
            scoring.map(|v| v.estimated_lap_time).unwrap_or(0.0)
        };
        if st.fuel_per_lap > 0.0 {
            st.laps_of_fuel = p.fuel / st.fuel_per_lap;
        }
        if st.ve_per_lap > 0.0 {
            st.laps_of_ve = p.virtual_energy / st.ve_per_lap;
        }

        let len = snap.session.track_length;
        let frac = if len > 0.0 { (p.lap_dist / len).clamp(0.0, 1.0) } else { 0.0 };
        let completed = scoring.map(|v| v.total_laps as i32).unwrap_or(0);
        let max_laps = snap.session.max_laps;
        let is_race = snap.session.kind == crate::model::SessionKind::Race;
        st.laps_remaining = if max_laps > 0 && max_laps < 10_000 && snap.session.end_et <= 0.0 {
            (max_laps - completed) as f64 - frac
        } else if st.avg_lap_time > 0.0 && snap.session.time_remaining > 0.0 {
            let laps = snap.session.time_remaining / st.avg_lap_time;
            // In a timed race the lap in progress when the clock hits zero is completed.
            if is_race { (laps + frac).ceil() - frac } else { laps }
        } else {
            0.0
        }
        .max(0.0);
        st.fuel_to_finish = st.laps_remaining * st.fuel_per_lap;
        st.ve_to_finish = st.laps_remaining * st.ve_per_lap;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drives the demo simulator through the recorder and checks the laps it produces.
    #[test]
    fn records_demo_laps() {
        let dir = std::env::temp_dir().join(format!("lmo-rec-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("r.db");
        let _ = std::fs::remove_file(&path);
        let db = Arc::new(Db::open(&path).unwrap());
        let (tx, rx) = std::sync::mpsc::channel();
        let mut rec = Recorder::new(db.clone(), tx);
        let mut sim = crate::demo::DemoSim::new();

        let mut last = None;
        for _ in 0..(60 * 60 * 6) {
            let mut snap = sim.step(1.0 / 60.0);
            rec.update(&mut snap);
            last = Some(snap);
        }
        let laps: Vec<NewLap> = rx.try_iter().collect();
        assert!(laps.len() >= 2, "expected laps, got {}", laps.len());
        let len = last.as_ref().unwrap().session.track_length;
        for lap in &laps {
            assert!(lap.lap_time > 60.0 && lap.lap_time < 200.0, "lap time {}", lap.lap_time);
            let [s1, s2, s3] = lap.sectors;
            let sum = s1.unwrap() + s2.unwrap() + s3.unwrap();
            assert!((sum - lap.lap_time).abs() < 1e-6);
            // ~30 Hz samples and distance covering the whole lap.
            let hz = lap.trace.len() as f64 / lap.lap_time;
            assert!(hz > 25.0 && hz < 35.0, "sample rate {hz}");
            let d_end = *lap.trace.d.last().unwrap() as f64;
            assert!(d_end > len * 0.97, "distance {d_end} of {len}");
            assert!(lap.fuel_used.unwrap() > 0.0);
        }
        let snap = last.unwrap();
        assert!(snap.timing.last_lap_time > 0.0);
        assert!(snap.strategy.fuel_per_lap > 0.0);
        assert!(snap.strategy.laps_remaining > 0.0);

        // Persisted laps become the PB reference for the live delta.
        for lap in &laps {
            db.insert_lap(lap).unwrap();
        }
        rec.invalidate_reference();
        let mut snap = sim.step(1.0 / 60.0);
        rec.update(&mut snap);
        assert!(snap.timing.personal_best > 0.0 || laps.iter().all(|l| !l.valid));
    }

    #[test]
    fn reference_interpolates_time_by_distance() {
        let mut trace = LapTrace::default();
        for i in 0..20 {
            trace.d.push(i as f32 * 10.0);
            trace.t.push(i as f32);
        }
        let r = Reference::from_trace(&trace).unwrap();
        assert_eq!(r.time_at(55.0), Some(5.5));
        assert_eq!(r.time_at(0.0), Some(0.0));
        assert_eq!(r.time_at(1000.0), None);
    }
}
