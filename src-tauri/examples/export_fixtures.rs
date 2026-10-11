//! Exports demo data as JSON fixtures for the browser UI preview (`pnpm dev`
//! outside Tauri). Usage: `cargo run --example export_fixtures -- <out_dir>`

use std::sync::Arc;

use lm_overlay_lib::db::{self, Db};
use lm_overlay_lib::demo::DemoSim;
use lm_overlay_lib::recorder::Recorder;
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
    for _ in 0..(60 * 60 * 7) {
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

    let sessions = db.list_sessions(None, None, 100, 0).unwrap();
    let laps = db.list_laps(sessions[0].session.id).unwrap();
    let traces: serde_json::Map<String, serde_json::Value> = laps
        .iter()
        .map(|l| (l.id.to_string(), serde_json::to_value(db.lap_trace(l.id).unwrap()).unwrap()))
        .collect();
    let fixtures = json!({
        "frames": frames,
        "sessions": sessions,
        "laps": laps,
        "traces": traces,
        "trackMap": db.track_map("Demo Circuit").unwrap(),
        "personalBests": db.personal_bests().unwrap(),
        "filters": db.filters().unwrap(),
    });
    std::fs::write(out.join("fixtures.json"), serde_json::to_string(&fixtures).unwrap()).unwrap();
    println!("{} frames, {} laps", frames.len(), laps.len());
}
