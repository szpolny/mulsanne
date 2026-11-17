//! Race classification: every car's finishing position, saved while a race
//! runs so a quit or crash still leaves the last known order.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::db::SessionRecord;
use crate::model::{SessionKind, Snapshot, Vehicle};

/// While the order keeps changing, it is saved at most this often (session seconds).
const SAVE_INTERVAL: f64 = 15.0;

/// First session phase of the race proper (green flag).
const PHASE_GREEN: u8 = 5;
/// The session is over and the results are final.
const PHASE_OVER: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FinishStatus {
    /// Still on track when the classification was saved.
    Running,
    Finished,
    Dnf,
    Dq,
}

impl FinishStatus {
    /// rF2 `mFinishStatus`: 0 none, 1 finished, 2 DNF, 3 DQ.
    pub fn from_game(v: i8) -> Self {
        match v {
            1 => Self::Finished,
            2 => Self::Dnf,
            3 => Self::Dq,
            _ => Self::Running,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Finished => "finished",
            Self::Dnf => "dnf",
            Self::Dq => "dq",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "finished" => Self::Finished,
            "dnf" => Self::Dnf,
            "dq" => Self::Dq,
            _ => Self::Running,
        }
    }
}

/// One car's line in the classification.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultEntry {
    pub position: i32,
    pub class_position: i32,
    pub driver: String,
    pub team: String,
    /// Entry name, e.g. "Toyota Gazoo Racing 2026 #7:LM".
    pub vehicle_name: String,
    /// Car model, e.g. "Toyota TR010 Hybrid"; empty when the game does not report it.
    pub car: String,
    pub car_class: String,
    pub number: String,
    /// Keys the game's render of this livery.
    pub veh_file: String,
    pub is_player: bool,
    pub laps: i32,
    pub best_lap: Option<f64>,
    /// Time behind the overall leader; only meaningful between cars on the same lap.
    pub gap: f64,
    pub pit_stops: i32,
    pub status: FinishStatus,
    /// Time from the green flag to taking the chequered flag; only for
    /// finishers whose start and finish were both seen.
    pub race_time: Option<f64>,
}

impl ResultEntry {
    pub fn from_vehicle(v: &Vehicle) -> Self {
        Self {
            position: v.place as i32,
            class_position: v.class_place as i32,
            driver: v.driver.clone(),
            team: v.team.clone(),
            vehicle_name: v.vehicle.clone(),
            car: v.model.clone(),
            car_class: v.class.clone(),
            number: v.number.clone(),
            veh_file: v.veh_file.clone(),
            is_player: v.is_player,
            laps: v.total_laps.max(0) as i32,
            best_lap: (v.best_lap > 0.0).then_some(v.best_lap),
            gap: v.time_behind_leader.max(0.0),
            pit_stops: v.num_pitstops.max(0) as i32,
            status: FinishStatus::from_game(v.finish_status),
            race_time: None,
        }
    }
}

/// A session's classification, ready to be stored.
#[derive(Debug, Clone)]
pub struct RaceResults {
    pub session: SessionRecord,
    /// The race was seen to the end; otherwise this is the last known order.
    pub complete: bool,
    pub entries: Vec<ResultEntry>,
}

/// Classification of the cars in a snapshot, in finishing order.
pub fn classify(snap: &Snapshot) -> Vec<ResultEntry> {
    classify_with(snap, &HashMap::new(), None)
}

/// Like [`classify`], with the session time each car took the chequered flag
/// where it was seen. Cars that finished on the winner's lap get their gap
/// from those times rather than from the game's live gap, and with the time
/// of the green flag, finishers get their race time.
fn classify_with(snap: &Snapshot, flag_et: &HashMap<i32, f64>, green_et: Option<f64>) -> Vec<ResultEntry> {
    let mut cars: Vec<&Vehicle> = snap.vehicles.iter().filter(|v| v.place > 0).collect();
    cars.sort_by_key(|v| v.place);
    let winner = cars.first().and_then(|v| flag_et.get(&v.id).map(|t| (*t, v.total_laps)));
    cars.into_iter()
        .map(|v| {
            let mut e = ResultEntry::from_vehicle(v);
            if let (Some(green), Some(et)) = (green_et, flag_et.get(&v.id)) {
                e.race_time = Some(et - green).filter(|t| *t > 0.0);
            }
            if let (Some((winner_et, laps)), Some(et)) = (winner, flag_et.get(&v.id)) {
                if v.total_laps == laps {
                    e.gap = (et - winner_et).max(0.0);
                }
            }
            e
        })
        .collect()
}

/// Follows race sessions and decides when their classification is saved.
#[derive(Default)]
pub struct ResultsTracker {
    current: Option<RaceResults>,
    /// The classification last handed out for saving.
    saved: Option<RaceResults>,
    saved_at: f64,
    /// Cars seen still racing, so a finish seen afterwards has a true time.
    running: HashSet<i32>,
    /// Session time each car was first seen to have finished.
    flag_et: HashMap<i32, f64>,
    /// The race the bookkeeping below belongs to.
    race_id: Option<i64>,
    /// The countdown was seen, so the green flag has a true time.
    seen_before_green: bool,
    green_et: Option<f64>,
}

/// What decides whether a save is due right away: each car's place, laps and status.
fn order(entries: &[ResultEntry]) -> Vec<(i32, i32, FinishStatus)> {
    entries.iter().map(|e| (e.position, e.laps, e.status)).collect()
}

impl ResultsTracker {
    /// Feeds one frame. Returns a classification to save, if one is due.
    /// `session` is the recorder's current session.
    pub fn update(&mut self, snap: &Snapshot, session: Option<&SessionRecord>) -> Option<RaceResults> {
        let racing = snap.session.kind == SessionKind::Race && !snap.vehicles.is_empty();
        let same_session = match (&self.current, session) {
            (Some(cur), Some(s)) => cur.session.id == s.id,
            _ => false,
        };
        if !racing || !same_session {
            // Left the race (or the game went away): save what is still unsaved.
            let flush = self.flush();
            self.current = None;
            self.saved = None;
            if flush.is_some() || !racing {
                return flush;
            }
        }
        let session = session?;
        let now = snap.session.current_et;
        if self.race_id != Some(session.id) {
            self.race_id = Some(session.id);
            self.running.clear();
            self.flag_et.clear();
            self.seen_before_green = false;
            self.green_et = None;
        }
        if snap.session.phase < PHASE_GREEN {
            self.seen_before_green = true;
        } else if self.seen_before_green && self.green_et.is_none() {
            self.green_et = Some(now);
        }
        for v in &snap.vehicles {
            match FinishStatus::from_game(v.finish_status) {
                FinishStatus::Running => {
                    self.running.insert(v.id);
                }
                FinishStatus::Finished if self.running.contains(&v.id) => {
                    self.flag_et.entry(v.id).or_insert(now);
                }
                _ => {}
            }
        }
        // Nothing worth keeping before the start or before anyone completed a lap.
        let started = snap.session.phase >= PHASE_GREEN || self.current.is_some();
        if !started || !snap.vehicles.iter().any(|v| v.total_laps > 0) {
            return None;
        }

        let entries = classify_with(snap, &self.flag_et, self.green_et);
        let player_done = entries.iter().any(|e| e.is_player && e.status != FinishStatus::Running);
        let complete = snap.session.phase == PHASE_OVER || player_done || self.current.as_ref().is_some_and(|c| c.complete);
        let due = match &self.saved {
            None => true,
            Some(saved) => {
                let (new, old) = (order(&entries), order(&saved.entries));
                let statuses_changed = new.len() != old.len() || new.iter().zip(&old).any(|(a, b)| a.2 != b.2);
                (complete && !saved.complete)
                    || statuses_changed
                    || (new != old && now - self.saved_at >= SAVE_INTERVAL)
                    // A restarted clock must not stall saving.
                    || now < self.saved_at
            }
        };
        self.current = Some(RaceResults { session: session.clone(), complete, entries });
        if !due {
            return None;
        }
        self.saved_at = now;
        self.flush()
    }

    /// The current classification, if it differs from the one last saved.
    fn flush(&mut self) -> Option<RaceResults> {
        let current = self.current.as_ref()?;
        let unchanged = self.saved.as_ref().is_some_and(|s| s.complete == current.complete && s.entries == current.entries);
        if unchanged {
            return None;
        }
        self.saved = Some(current.clone());
        self.saved.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Session, Source};

    fn record(id: i64) -> SessionRecord {
        SessionRecord {
            id,
            started_at: 1,
            source: "lmu".into(),
            track: "T".into(),
            track_length: 1000.0,
            session_type: "Race".into(),
            car: "Car".into(),
            vehicle_name: "#1 Car".into(),
            veh_file: String::new(),
            car_class: "GT3".into(),
            driver: "Me".into(),
            game_version: None,
        }
    }

    fn car(id: i32, place: u8, laps: i16, player: bool) -> Vehicle {
        Vehicle { id, place, class_place: place as u16, total_laps: laps, is_player: player, driver: format!("D{id}"), ..Default::default() }
    }

    fn snap(phase: u8, et: f64, vehicles: Vec<Vehicle>) -> Snapshot {
        Snapshot {
            source: Source::Lmu,
            session: Session { kind: SessionKind::Race, code: 10, phase, current_et: et, ..Default::default() },
            vehicles,
            ..Default::default()
        }
    }

    #[test]
    fn saves_order_changes_throttled_and_the_finish_at_once() {
        let mut t = ResultsTracker::default();
        let s = record(1);
        // Before the start nothing is saved.
        assert!(t.update(&snap(4, 0.0, vec![car(1, 1, 0, true), car(2, 2, 0, false)]), Some(&s)).is_none());
        // First lap done: saved as incomplete.
        let r = t.update(&snap(5, 100.0, vec![car(1, 1, 1, true), car(2, 2, 1, false)]), Some(&s)).unwrap();
        assert!(!r.complete);
        assert_eq!(r.entries.len(), 2);
        // Same order: nothing new.
        assert!(t.update(&snap(5, 101.0, vec![car(1, 1, 1, true), car(2, 2, 1, false)]), Some(&s)).is_none());
        // An overtake is saved, but not more often than the interval.
        assert!(t.update(&snap(5, 105.0, vec![car(1, 2, 1, true), car(2, 1, 1, false)]), Some(&s)).is_none());
        let r = t.update(&snap(5, 116.0, vec![car(1, 2, 2, true), car(2, 1, 2, false)]), Some(&s)).unwrap();
        assert_eq!(r.entries[0].driver, "D2");
        // The player taking the flag completes the result straight away.
        let mut done = car(1, 2, 3, true);
        done.finish_status = 1;
        let r = t.update(&snap(5, 120.0, vec![done.clone(), car(2, 1, 3, false)]), Some(&s)).unwrap();
        assert!(r.complete);
        assert_eq!(r.entries[1].status, FinishStatus::Finished);
        // Unsaved changes are saved when the session ends.
        let mut other = car(2, 1, 3, false);
        other.finish_status = 1;
        assert!(t.update(&snap(5, 121.0, vec![done.clone(), other.clone()]), Some(&s)).is_some());
        // A late best lap update alone waits for the next save.
        let mut best_only = done.clone();
        best_only.best_lap = 95.5;
        assert!(t.update(&snap(8, 122.0, vec![best_only, other]), Some(&s)).is_none());
        let r = t.update(&Snapshot::default(), Some(&s)).unwrap();
        assert_eq!(r.entries[1].best_lap, Some(95.5));
        assert!(t.update(&Snapshot::default(), Some(&s)).is_none());
    }

    #[test]
    fn finishing_gaps_come_from_the_flag_only_when_the_finish_was_seen() {
        let finished = |id: i32, place: u8, behind: f64| {
            let mut v = car(id, place, 5, id == 1);
            v.finish_status = 1;
            v.time_behind_leader = behind;
            v
        };
        // Joined after both had finished: the game's gaps are kept.
        let mut t = ResultsTracker::default();
        let r = t.update(&snap(8, 500.0, vec![finished(2, 1, 0.0), finished(1, 2, 4.5)]), Some(&record(1))).unwrap();
        assert_eq!(r.entries[1].gap, 4.5);

        assert_eq!(r.entries[1].race_time, None);

        // Seen crossing the line: the gap is the time between the two crossings.
        // Joined after the green flag, so the race time is unknown.
        let mut t = ResultsTracker::default();
        t.update(&snap(5, 480.0, vec![car(2, 1, 4, false), car(1, 2, 4, true)]), Some(&record(2)));
        t.update(&snap(5, 490.0, vec![finished(2, 1, 0.0), car(1, 2, 4, true)]), Some(&record(2)));
        let r = t.update(&snap(5, 493.25, vec![finished(2, 1, 0.0), finished(1, 2, 0.0)]), Some(&record(2))).unwrap();
        assert_eq!(r.entries[1].gap, 3.25);
        assert_eq!(r.entries[1].race_time, None);

        // From the countdown: race times run from the green flag.
        let mut t = ResultsTracker::default();
        t.update(&snap(4, 10.0, vec![car(2, 1, 0, false), car(1, 2, 0, true)]), Some(&record(3)));
        t.update(&snap(5, 12.0, vec![car(2, 1, 0, false), car(1, 2, 0, true)]), Some(&record(3)));
        t.update(&snap(5, 480.0, vec![car(2, 1, 4, false), car(1, 2, 4, true)]), Some(&record(3)));
        t.update(&snap(5, 490.0, vec![finished(2, 1, 0.0), car(1, 2, 4, true)]), Some(&record(3)));
        let r = t.update(&snap(5, 493.25, vec![finished(2, 1, 0.0), finished(1, 2, 0.0)]), Some(&record(3))).unwrap();
        assert_eq!((r.entries[0].race_time, r.entries[1].race_time), (Some(478.0), Some(481.25)));
    }

    #[test]
    fn other_sessions_are_ignored_and_a_new_race_starts_fresh() {
        let mut t = ResultsTracker::default();
        let mut practice = snap(5, 100.0, vec![car(1, 1, 3, true)]);
        practice.session.kind = SessionKind::Practice;
        assert!(t.update(&practice, Some(&record(1))).is_none());

        assert!(t.update(&snap(5, 100.0, vec![car(1, 1, 1, true)]), Some(&record(2))).is_some());
        assert!(t.update(&snap(5, 101.0, vec![car(1, 1, 1, true)]), Some(&record(2))).is_none());
        // A restart is a new session: its first lap is saved under the new id.
        let r = t.update(&snap(5, 50.0, vec![car(1, 1, 1, true)]), Some(&record(3))).unwrap();
        assert_eq!(r.session.id, 3);
        assert!(!r.complete);
    }
}
