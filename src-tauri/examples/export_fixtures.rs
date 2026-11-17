//! Exports demo data as JSON fixtures for the browser UI preview (`pnpm dev`
//! outside Tauri). Usage: `cargo run --example export_fixtures -- <out_dir>`

use std::sync::Arc;

use lm_overlay_lib::db::{self, Db};
use lm_overlay_lib::demo::DemoSim;
use lm_overlay_lib::recorder::Recorder;
use lm_overlay_lib::results::{self, FinishStatus, RaceResults};
use serde_json::json;

fn main() {
    let out = std::path::PathBuf::from(std::env::args().nth(1).expect("output dir"));
    std::fs::create_dir_all(&out).unwrap();
    let db_path = std::env::temp_dir().join("lmo-fixtures.db");
    let _ = std::fs::remove_file(&db_path);
    let db = Arc::new(Db::open(&db_path).unwrap());
    let (tx, rx) = std::sync::mpsc::channel();
    let mut rec = Recorder::new(db.clone(), tx);
    let mut sim = DemoSim::new();

    // Six laps, then 20 s of snapshots at 20 Hz for the live view.
    let dt = 1.0 / 60.0;
    for _ in 0..(60 * 60 * 24) {
        let mut s = sim.step(dt);
        rec.update(&mut s);
        for lap in rx.try_iter() {
            let id = db.insert_lap(&lap).unwrap();
            if lap.valid && db.track_map(&lap.session.track).unwrap().is_none() {
                db.save_track_map(&db::track_map_from_trace(&lap.session.track, lap.session.track_length, &lap.trace, 5.0), id).unwrap();
            }
        }
    }
    let mut frames = Vec::new();
    for i in 0..(60 * 20) {
        let mut s = sim.step(dt);
        rec.update(&mut s);
        if i % 3 == 0 {
            s.frame = i as u64;
            frames.push(s);
        }
    }

    let practice = db.list_sessions(None, None, 100, 0).unwrap();
    let laps = db.list_laps(practice[0].session.id).unwrap();
    let personal_bests = db.personal_bests().unwrap();

    // A finished race at the same track, for the results table. The demo only
    // simulates practice, so its last frame stands in for the final order.
    let last = frames.last().unwrap();
    let mut entries = results::classify(last);
    let leader_laps = entries.first().map_or(0, |e| e.laps);
    for e in &mut entries {
        e.status = FinishStatus::Finished;
        // Mid-lap, cars just behind the leader have not crossed the line yet; at the flag they would have.
        if let Some(v) = last.vehicles.iter().find(|v| v.place as i32 == e.position) {
            e.laps = leader_laps - v.laps_behind_leader;
        }
        if e.is_player {
            e.veh_file = last.player.as_ref().map(|p| p.veh_file.clone()).unwrap_or_default();
        }
    }
    // The demo's laps take ~110 s; the leader took the flag on the last frame.
    let leader_time = last.session.current_et;
    for e in &mut entries {
        if let Some(v) = last.vehicles.iter().find(|v| v.place as i32 == e.position) {
            e.race_time = Some(leader_time + (v.time_behind_leader - v.laps_behind_leader as f64 * 110.0).max(0.0) + e.position as f64 * 0.05);
        }
    }
    if let Some(e) = entries.iter_mut().rev().find(|e| !e.is_player) {
        e.race_time = None;
        e.status = FinishStatus::Dnf;
        e.laps = (e.laps - 2).max(0);
    }
    let race = lm_overlay_lib::db::SessionRecord {
        id: practice[0].session.id + 3_600_000,
        started_at: practice[0].session.started_at + 3600,
        session_type: "Race".into(),
        ..practice[0].session.clone()
    };
    db.save_results(&RaceResults { session: race.clone(), complete: true, entries }).unwrap();
    let race_results: serde_json::Map<String, serde_json::Value> =
        [(race.id.to_string(), serde_json::to_value(db.session_results(race.id).unwrap()).unwrap())].into_iter().collect();
    // The race shows the practice laps too, so its summary counts them.
    let sessions: Vec<_> = db
        .list_sessions(None, None, 100, 0)
        .unwrap()
        .into_iter()
        .map(|mut s| {
            if s.session.id == race.id {
                s.lap_count = practice[0].lap_count;
                s.valid_laps = practice[0].valid_laps;
                s.best_lap = practice[0].best_lap;
            }
            s
        })
        .collect();
    let traces: serde_json::Map<String, serde_json::Value> = laps
        .iter()
        .map(|l| (l.id.to_string(), serde_json::to_value(db.lap_trace(l.id).unwrap()).unwrap()))
        .collect();
    let fixtures = json!({
        "frames": frames,
        "sessions": sessions,
        "laps": laps,
        "traces": traces,
        "results": race_results,
        "trackMap": db.track_map(lm_overlay_lib::demo::TRACK_NAME).unwrap(),
        "personalBests": personal_bests,
        "filters": db.filters().unwrap(),
    });
    std::fs::write(out.join("fixtures.json"), serde_json::to_string(&fixtures).unwrap()).unwrap();

    // One shared live timing frame and map, for previewing mulsanne.xyz/live
    let track_map = db.track_map(lm_overlay_lib::demo::TRACK_NAME).unwrap().expect("demo track map");
    let live = json!({
        "frame": lm_overlay_lib::live_share::frame(frames.last().unwrap()),
        "map": lm_overlay_lib::live_share::map(&track_map),
    });
    std::fs::write(out.join("live.json"), serde_json::to_string(&live).unwrap()).unwrap();
    println!("{} frames, {} laps", frames.len(), laps.len());
}
