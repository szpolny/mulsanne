//! Penalty details for every car. Shared memory only has a per-car count, but
//! the game writes each penalty it hands out to its trace log
//! (`UserData/Log/trace_*.txt`), including those for remote cars online:
//!
//! `score.cpp 1372: Local penalty et=1385.6 0 10 0 0 "Speeding In Pitlane"`
//!
//! The line has no driver, so it is matched to the car whose count went up at
//! the same session time.

use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::model::{Penalty, PenaltyKind, Snapshot};

/// Max session-time difference between a log line and a count change.
const MATCH_WINDOW: f64 = 5.0;
/// Log lines that matched no car within this long are dropped.
const EVENT_TTL: f64 = 60.0;
const POLL_EVERY: Duration = Duration::from_millis(500);
const RESCAN_EVERY: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq)]
struct Event {
    et: f64,
    penalty: Penalty,
}

/// One unserved penalty: when the count went up (unknown for penalties the
/// car already had when first seen) and its details once matched.
struct Slot {
    added_et: Option<f64>,
    penalty: Option<Penalty>,
}

#[derive(Default)]
pub struct PenaltyTracker {
    log: TraceLog,
    events: Vec<Event>,
    cars: HashMap<i32, VecDeque<Slot>>,
    session: Option<(i32, f64)>,
}

impl PenaltyTracker {
    /// Fills `Vehicle::penalties` from the trace log next to the game that owns `hwnd`.
    pub fn update(&mut self, snap: &mut Snapshot, hwnd: u64) {
        let et = snap.session.current_et;
        let code = snap.session.code;
        // A new session restarts the clock: forget everything from the old one.
        if self.session.is_some_and(|(c, last)| c != code || et + 1.0 < last) {
            self.events.clear();
            self.cars.clear();
        }
        self.session = Some((code, et));

        self.events.extend(self.log.poll(hwnd));
        self.events.retain(|e| e.et > et - EVENT_TTL);

        let first_seen = self.cars.is_empty();
        for v in &mut snap.vehicles {
            let count = v.num_penalties.max(0) as usize;
            let slots = self.cars.entry(v.id).or_default();
            // Served penalties come off the front.
            while slots.len() > count {
                slots.pop_front();
            }
            while slots.len() < count {
                let added_et = (!first_seen && et > 0.0).then_some(et);
                slots.push_back(Slot { added_et, penalty: None });
            }
            for slot in slots.iter_mut().filter(|s| s.penalty.is_none()) {
                let Some(added) = slot.added_et else { continue };
                let best = self
                    .events
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| (e.et - added).abs() <= MATCH_WINDOW)
                    .min_by(|(_, a), (_, b)| (a.et - added).abs().total_cmp(&(b.et - added).abs()))
                    .map(|(i, _)| i);
                if let Some(i) = best {
                    slot.penalty = Some(self.events.remove(i).penalty);
                }
            }
            v.penalties = slots.iter().filter_map(|s| s.penalty.clone()).collect();
        }
    }
}

/// Follows the newest trace file, returning only lines written after it was first opened.
#[derive(Default)]
struct TraceLog {
    hwnd: u64,
    dir: Option<PathBuf>,
    path: Option<PathBuf>,
    offset: u64,
    partial: Vec<u8>,
    last_poll: Option<Instant>,
    last_scan: Option<Instant>,
}

impl TraceLog {
    fn poll(&mut self, hwnd: u64) -> Vec<Event> {
        if self.last_poll.is_some_and(|t| t.elapsed() < POLL_EVERY) {
            return Vec::new();
        }
        self.last_poll = Some(Instant::now());
        if hwnd != self.hwnd {
            self.hwnd = hwnd;
            self.dir = super::shm::process_dir(hwnd).map(|d| d.join("UserData").join("Log"));
            self.last_scan = None;
        }
        if self.last_scan.is_none_or(|t| t.elapsed() >= RESCAN_EVERY) {
            self.last_scan = Some(Instant::now());
            let newest = self.dir.as_deref().and_then(newest_trace);
            if newest != self.path {
                // Start at the end: earlier lines belong to sessions we can't match against.
                self.offset = newest.as_deref().and_then(|p| p.metadata().ok()).map_or(0, |m| m.len());
                self.partial.clear();
                self.path = newest;
            }
        }
        let Some(path) = &self.path else { return Vec::new() };
        let Ok(mut file) = File::open(path) else { return Vec::new() };
        let len = file.metadata().map_or(0, |m| m.len());
        if len < self.offset {
            self.offset = 0;
            self.partial.clear();
        }
        if len == self.offset || file.seek(SeekFrom::Start(self.offset)).is_err() {
            return Vec::new();
        }
        let mut data = std::mem::take(&mut self.partial);
        let Ok(read) = file.read_to_end(&mut data) else { return Vec::new() };
        self.offset += read as u64;
        // Keep an unfinished last line for the next poll.
        let complete = data.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        self.partial = data.split_off(complete);
        String::from_utf8_lossy(&data).lines().filter_map(parse_line).collect()
    }
}

fn newest_trace(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_ascii_lowercase();
            name.starts_with("trace_") && name.ends_with(".txt")
        })
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

/// Parses `... Local penalty et=980.6 0 10 0 0 "Exiting Pits Under Red"`.
/// "Queued local penalty" lines (during formation) are skipped: the game logs
/// "Local penalty" again when it actually hands the penalty out.
fn parse_line(line: &str) -> Option<Event> {
    let rest = line.split_once(": Local penalty et=")?.1;
    let (fields, reason) = rest.split_once('"')?;
    let reason = reason.trim_end().trim_end_matches('"');
    let mut fields = fields.split_whitespace();
    let et: f64 = fields.next()?.parse().ok()?;
    let kind = match fields.next()?.parse::<i32>().ok()? {
        0 => PenaltyKind::StopGo,
        1 => PenaltyKind::DriveThrough,
        _ => PenaltyKind::Other,
    };
    let seconds = fields.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    Some(Event {
        et,
        penalty: Penalty {
            kind,
            seconds: if kind == PenaltyKind::StopGo { seconds } else { 0 },
            reason: reason.to_string(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Vehicle;

    fn event(et: f64, kind: PenaltyKind, seconds: i32, reason: &str) -> Event {
        Event { et, penalty: Penalty { kind, seconds, reason: reason.into() } }
    }

    #[test]
    fn parses_log_lines() {
        assert_eq!(
            parse_line(r#"3913.39s score.cpp    1372: Local penalty et=1385.6 0 10 0 0 "Speeding In Pitlane""#),
            Some(event(1385.6, PenaltyKind::StopGo, 10, "Speeding In Pitlane"))
        );
        assert_eq!(
            parse_line(r#"1871.89s score.cpp    1372: Local penalty et=132.9 1 0 0 0 "Speeding""#),
            Some(event(132.9, PenaltyKind::DriveThrough, 0, "Speeding"))
        );
        assert_eq!(parse_line(r#"1819.89s score.cpp    1368: Queued local penalty et=80.9 1 0 0 0 "Speeding""#), None);
        assert_eq!(parse_line("5056.17s CoherentReso  383: OnHTTPResourceRequest failed"), None);
    }

    fn snap(et: f64, counts: &[(i32, i16)]) -> Snapshot {
        let mut s = Snapshot::default();
        s.session.current_et = et;
        s.vehicles = counts.iter().map(|&(id, n)| Vehicle { id, num_penalties: n, ..Default::default() }).collect();
        s
    }

    #[test]
    fn matches_events_to_cars_by_session_time() {
        let mut t = PenaltyTracker::default();
        t.update(&mut snap(100.0, &[(1, 0), (2, 0)]), 0);

        // Car 2 gets a penalty; the log line arrives a moment later.
        let mut s = snap(132.9, &[(1, 0), (2, 1)]);
        t.update(&mut s, 0);
        assert!(s.vehicles[1].penalties.is_empty());
        t.events.push(event(132.9, PenaltyKind::DriveThrough, 0, "Speeding"));
        let mut s = snap(133.4, &[(1, 0), (2, 1)]);
        t.update(&mut s, 0);
        assert!(s.vehicles[0].penalties.is_empty());
        assert_eq!(s.vehicles[1].penalties[0].reason, "Speeding");

        // Served: the detail goes with it.
        let mut s = snap(200.0, &[(1, 0), (2, 0)]);
        t.update(&mut s, 0);
        assert!(s.vehicles[1].penalties.is_empty());
    }

    #[test]
    fn ignores_penalties_from_before_tracking_started() {
        let mut t = PenaltyTracker::default();
        t.events.push(event(50.0, PenaltyKind::StopGo, 10, "Speeding In Pitlane"));
        let mut s = snap(52.0, &[(1, 1)]);
        t.update(&mut s, 0);
        assert!(s.vehicles[0].penalties.is_empty());
    }

    #[test]
    fn new_session_drops_old_events() {
        let mut t = PenaltyTracker::default();
        t.update(&mut snap(900.0, &[(1, 0)]), 0);
        t.events.push(event(120.0, PenaltyKind::StopGo, 10, "Old"));
        t.update(&mut snap(10.0, &[(1, 0)]), 0);
        let mut s = snap(120.0, &[(1, 1)]);
        t.update(&mut s, 0);
        assert!(s.vehicles[0].penalties.is_empty());
    }
}
