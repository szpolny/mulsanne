//! Live timing sharing: while a share is open, the latest snapshot is sent
//! to the Mulsanne server once a second, and anyone with the link watches it
//! on `https://mulsanne.xyz/live#<id>`.
//!
//! Sharing is part of the online features: it needs them turned on and uses
//! the same signed identity. It sends the session, the whole field's timing
//! and positions, the player's car status, and the track outline, as
//! [`frame`] and [`map`] build them. The viewer page in `mulsanne-web` reads
//! the same shape (`src/lib/live.ts` there); bump [`FRAME_VERSION`] when
//! changing it incompatibly.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::db::TrackMap;
use crate::model::{Snapshot, Source, Vehicle};
use crate::online::{self, RequestError};
use crate::AppState;

const VIEWER_URL: &str = "https://mulsanne.xyz/live";
const FRAME_VERSION: u32 = 1;
const INTERVAL: Duration = Duration::from_secs(1);
/// How often to look for a recorded track map while none has been sent.
const MAP_RETRY: Duration = Duration::from_secs(15);
/// Track maps are stored every ~5 m; viewers need far fewer points.
const MAP_STRIDE: usize = 4;
pub const EVENT: &str = "live-share-changed";

/// What the companion shows about the open share.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareStatus {
    pub id: String,
    pub url: String,
    /// Unix seconds.
    pub expires_at: i64,
    pub viewers: usize,
    /// The last push failed; cleared by the next one that succeeds.
    pub error: Option<String>,
    /// The server ended the share (expired, or started elsewhere); nothing
    /// more is sent.
    pub ended: bool,
}

#[derive(Deserialize)]
struct Created {
    id: String,
    expires_at: i64,
}

#[derive(Deserialize)]
struct Pushed {
    viewers: usize,
    has_map: bool,
}

struct Running {
    status: Arc<Mutex<ShareStatus>>,
    task: tauri::async_runtime::JoinHandle<()>,
}

/// The open share, if any.
#[derive(Default)]
pub struct LiveShare {
    running: Mutex<Option<Running>>,
}

impl LiveShare {
    pub fn status(&self) -> Option<ShareStatus> {
        let running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        running.as_ref().map(|r| r.status.lock().unwrap_or_else(|e| e.into_inner()).clone())
    }

    fn take(&self) -> Option<Running> {
        self.running.lock().unwrap_or_else(|e| e.into_inner()).take()
    }

    /// Takes the open share only if it is the one with `status`.
    fn take_own(&self, status: &Arc<Mutex<ShareStatus>>) -> Option<Running> {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        if running.as_ref().is_some_and(|r| Arc::ptr_eq(&r.status, status)) {
            running.take()
        } else {
            None
        }
    }
}

fn path(id: &str, rest: &str) -> String {
    format!("/v1/live/{id}{rest}")
}

/// Opens a new share (ending the previous one) and starts sending.
pub async fn start(app: &AppHandle) -> Result<ShareStatus, String> {
    let state = app.state::<AppState>();
    online::require_enabled(&state.config_path)?;
    let key = online::require_key(&state.data_dir)?;
    stop(app).await;

    let created: Created = online::request_some(Some(&key), Method::POST, "/v1/live", None).await?;
    let status = ShareStatus {
        url: format!("{VIEWER_URL}#{}", created.id),
        id: created.id,
        expires_at: created.expires_at,
        ..Default::default()
    };
    let shared = Arc::new(Mutex::new(status.clone()));
    let task = tauri::async_runtime::spawn(run(app.clone(), key, shared.clone()));
    if let Some(old) = state.live.running.lock().unwrap_or_else(|e| e.into_inner()).replace(Running { status: shared, task }) {
        // Another start raced this one; the server already ended its share.
        old.task.abort();
    }
    let _ = app.emit(EVENT, Some(&status));
    Ok(status)
}

/// Stops sending and ends the share on the server, so viewers see it end.
pub async fn stop(app: &AppHandle) {
    let Some(running) = app.state::<AppState>().live.take() else { return };
    running.task.abort();
    end(app, &running.status).await;
}

/// Tells the companion and the server that the share is over.
async fn end(app: &AppHandle, status: &Mutex<ShareStatus>) {
    let status = status.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let _ = app.emit(EVENT, None::<ShareStatus>);
    if status.ended {
        return;
    }
    if let Ok(key) = online::require_key(&app.state::<AppState>().data_dir) {
        // Best effort: if this fails, the server ends it after a few idle minutes.
        let _ = online::request::<Value>(Some(&key), Method::DELETE, &path(&status.id, ""), None).await;
    }
}

async fn run(app: AppHandle, key: ed25519_dalek::SigningKey, status: Arc<Mutex<ShareStatus>>) {
    let id = status.lock().unwrap_or_else(|e| e.into_inner()).id.clone();
    let update = |f: &dyn Fn(&mut ShareStatus)| {
        let mut s = status.lock().unwrap_or_else(|e| e.into_inner());
        let before = s.clone();
        f(&mut s);
        if *s != before {
            let _ = app.emit(EVENT, Some(&*s));
        }
    };

    // The track whose map the server has, and when we last looked for one.
    let mut map_sent: Option<String> = None;
    let mut map_checked: Option<(String, Instant)> = None;
    let mut tick = tokio::time::interval(INTERVAL);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tick.tick().await;
        let state = app.state::<AppState>();
        if !online::settings(&state.config_path).enabled {
            // Online features were turned off: stop sharing. Not through
            // `stop`, which would abort this task mid-request.
            if let Some(own) = state.live.take_own(&status) {
                end(&app, &own.status).await;
            }
            return;
        }
        let snap = state.shared.latest.read().map(|s| s.clone()).unwrap_or_default();

        let body = frame(&snap);
        match online::request_some::<Pushed>(Some(&key), Method::PUT, &path(&id, "/frame"), Some(&body)).await {
            Ok(p) => {
                if !p.has_map {
                    map_sent = None;
                }
                update(&|s| {
                    s.viewers = p.viewers;
                    s.error = None;
                });
            }
            Err(RequestError::Status(StatusCode::NOT_FOUND, _)) => {
                update(&|s| {
                    s.ended = true;
                    s.viewers = 0;
                    s.error = Some("This live session has ended. Start a new one to share again.".into());
                });
                return;
            }
            Err(RequestError::Status(StatusCode::TOO_MANY_REQUESTS, _)) => {}
            Err(e) => {
                let message = e.to_string();
                update(&|s| s.error = Some(message.clone()));
            }
        }

        let track = &snap.session.track;
        let due = match &map_checked {
            Some((t, at)) => t != track || at.elapsed() >= MAP_RETRY,
            None => true,
        };
        if !track.is_empty() && map_sent.as_deref() != Some(track) && due {
            map_checked = Some((track.clone(), Instant::now()));
            if let Ok(Some(m)) = state.db.track_map(track) {
                let body = map(&m);
                if online::request::<Value>(Some(&key), Method::PUT, &path(&id, "/map"), Some(&body)).await.is_ok() {
                    map_sent = Some(track.clone());
                }
            }
        }
    }
}

/// Rounds to `places` decimals, which keeps frames small.
fn r(x: f64, places: i32) -> f64 {
    if !x.is_finite() {
        return 0.0;
    }
    let p = 10f64.powi(places);
    (x * p).round() / p
}

fn opt(x: Option<f64>, places: i32) -> Option<f64> {
    x.filter(|v| v.is_finite()).map(|v| r(v, places))
}

fn vehicle(v: &Vehicle) -> Value {
    json!({
        "id": v.id,
        "place": v.place,
        "classPlace": v.class_place,
        "driver": v.driver,
        "number": v.number,
        "vehicle": v.vehicle,
        "model": v.model,
        "team": v.team,
        "class": v.class,
        "isPlayer": v.is_player,
        "laps": v.total_laps,
        "lapDist": r(v.lap_dist, 0),
        "sector": v.sector,
        "bestLap": r(v.best_lap, 3),
        "lastLap": r(v.last_lap, 3),
        "gapNext": r(v.time_behind_next, 3),
        "lapsNext": v.laps_behind_next,
        "gapLeader": r(v.time_behind_leader, 3),
        "lapsLeader": v.laps_behind_leader,
        "inPits": v.in_pits,
        "pitState": v.pit_state,
        "inGarage": v.in_garage,
        "pitstops": v.num_pitstops,
        "penalties": v.num_penalties,
        "finishStatus": v.finish_status,
        "flag": v.flag,
        "x": r(v.pos[0], 1),
        "z": r(v.pos[2], 1),
        "tyre": v.tyre_compounds[0],
        "tyreWear": opt(v.tyre_wear, 3),
        "tyreLaps": v.tyre_laps,
        "energy": opt(v.virtual_energy, 3),
    })
}

/// One frame for viewers. See the module docs.
pub fn frame(snap: &Snapshot) -> Value {
    let s = &snap.session;
    let w = &s.weather;
    let player = snap.player.as_ref().map(|p| {
        let tyres: Vec<Value> = p
            .wheels
            .iter()
            .map(|w| {
                json!({
                    "wear": r(w.wear, 3),
                    "temp": r(w.surface_temp.iter().sum::<f64>() / 3.0, 1),
                    "carcass": r(w.carcass_temp, 1),
                    "optimal": r(w.optimal_temp, 1),
                    "pressure": r(w.pressure, 1),
                    "brake": r(w.brake_temp, 0),
                    "compound": w.compound,
                    "flat": w.flat,
                    "detached": w.detached,
                })
            })
            .collect();
        json!({
            "id": p.id,
            "vehicleName": p.vehicle_name,
            "vehicleModel": p.vehicle_model,
            "vehicleClass": p.vehicle_class,
            "lap": p.lap_number,
            "lapDist": r(p.lap_dist, 0),
            "inPits": p.in_pits,
            "speed": r(p.speed, 1),
            "gear": p.gear,
            "fuel": r(p.fuel, 2),
            "fuelCapacity": r(p.fuel_capacity, 1),
            "energy": r(p.virtual_energy, 4),
            "battery": r(p.battery, 3),
            "deltaBest": r(p.delta_best, 3),
            "lapInvalidated": p.lap_invalidated,
            "trackLimits": p.track_limits_steps,
            "trackLimitsPerPenalty": p.track_limits_steps_per_penalty,
            "tc": p.tc,
            "abs": p.abs,
            "motorMap": p.motor_map,
            "brakeBias": r(p.brake_bias_front, 3),
            "waterTemp": r(p.water_temp, 1),
            "oilTemp": r(p.oil_temp, 1),
            "overheating": p.overheating,
            "detached": p.detached,
            "dents": p.dents,
            "damage": p.damage.as_ref().map(|d| json!({
                "aero": r(d.aero, 3),
                "suspension": d.suspension.map(|x| r(x, 3)),
            })),
            "tyres": tyres,
        })
    });
    let t = &snap.timing;
    let st = &snap.strategy;
    json!({
        "v": FRAME_VERSION,
        "source": match snap.source { Source::Lmu => "lmu", Source::Demo => "demo", Source::None => "none" },
        "connected": snap.connected,
        "inRealtime": snap.in_realtime,
        "session": {
            "track": s.track,
            "trackLength": r(s.track_length, 1),
            "kind": s.kind,
            "phase": s.phase,
            "currentEt": r(s.current_et, 1),
            "endEt": r(s.end_et, 1),
            "timeRemaining": r(s.time_remaining, 1),
            "maxLaps": s.max_laps,
            "timeOfDay": r(s.time_of_day, 0),
            "yellowFlagState": s.yellow_flag_state,
            "sectorYellow": s.sector_yellow,
            "startLight": s.start_light,
            "redLights": s.num_red_lights,
            "playerName": s.player_name,
            "weather": {
                "ambientTemp": r(w.ambient_temp, 1),
                "trackTemp": r(w.track_temp, 1),
                "raining": r(w.raining, 2),
                "cloudCoverage": w.cloud_coverage,
                "windSpeed": r(w.wind_speed, 1),
                "windDirection": r(w.wind_direction, 2),
                "wetness": r(w.avg_wetness, 2),
                "grip": w.grip_level,
            },
        },
        "player": player,
        "timing": {
            "currentLap": r(t.current_lap_time, 2),
            "lastLap": r(t.last_lap_time, 3),
            "lastLapValid": t.last_lap_valid,
            "sessionBest": r(t.session_best, 3),
            "personalBest": r(t.personal_best, 3),
            "deltaPb": opt(t.delta_pb, 3),
            "predictedLap": opt(t.predicted_lap, 3),
            "stintLaps": t.stint_laps,
            "stintTime": r(t.stint_time, 0),
            "lastSectors": t.last_sectors.map(|x| r(x, 3)),
            "bestSectors": t.session_best_sectors.map(|x| r(x, 3)),
        },
        "strategy": {
            "fuelPerLap": r(st.fuel_per_lap, 3),
            "energyPerLap": r(st.ve_per_lap, 4),
            "lapsOfFuel": r(st.laps_of_fuel, 2),
            "lapsOfEnergy": r(st.laps_of_ve, 2),
            "lapsRemaining": r(st.laps_remaining, 2),
            "fuelToFinish": r(st.fuel_to_finish, 2),
            "energyToFinish": r(st.ve_to_finish, 4),
            "avgLapTime": r(st.avg_lap_time, 3),
            "tyreWearPerLap": st.tyre_wear_per_lap.map(|x| r(x, 4)),
        },
        "vehicles": snap.vehicles.iter().map(vehicle).collect::<Vec<_>>(),
    })
}

/// The track outline for viewers, as `[x, z]` world coordinates.
pub fn map(m: &TrackMap) -> Value {
    let mut points: Vec<[f64; 2]> =
        m.points.iter().step_by(MAP_STRIDE).map(|p| [r(p[0] as f64, 1), r(p[1] as f64, 1)]).collect();
    // Keep the last point so a closed outline stays closed.
    if let Some(last) = m.points.last() {
        if m.points.len() % MAP_STRIDE != 1 {
            points.push([r(last[0] as f64, 1), r(last[1] as f64, 1)]);
        }
    }
    json!({ "track": m.track, "length": r(m.length, 1), "points": points })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo::DemoSim;

    #[test]
    fn frames_fit_the_server_limit_with_a_full_grid() {
        let mut sim = DemoSim::new();
        let mut snap = sim.step(0.1);
        for _ in 0..200 {
            snap = sim.step(0.1);
        }
        // A full LMU grid with long names, well past the demo's field.
        let template = snap.vehicles[0].clone();
        snap.vehicles = (0..70)
            .map(|i| Vehicle {
                id: i,
                driver: "Maximilian Alexander Featherstonehaugh-Wright".into(),
                team: "Very Long Endurance Racing Team Name Motorsport".into(),
                ..template.clone()
            })
            .collect();
        let bytes = frame(&snap).to_string().len();
        // The server accepts 64 KiB; keep a quarter of it spare.
        assert!(bytes < 48 * 1024, "frame is {bytes} bytes");
        assert!(snap.player.is_some());
        assert_eq!(frame(&snap)["v"], FRAME_VERSION);
    }

    #[test]
    fn map_is_thinned_and_stays_closed() {
        let points: Vec<[f32; 2]> = (0..10).map(|i| [i as f32, 0.0]).collect();
        let m = TrackMap { track: "T".into(), length: 50.0, dist: vec![0.0; 10], points };
        let v = map(&m);
        let pts = v["points"].as_array().unwrap();
        assert_eq!(pts.first().unwrap(), &json!([0.0, 0.0]));
        assert_eq!(pts.last().unwrap(), &json!([9.0, 0.0]));
        assert_eq!(pts.len(), 4); // 0, 4, 8, then the last point
    }

    #[test]
    fn rounding_drops_noise_and_non_finite_values() {
        assert_eq!(r(1.23456, 2), 1.23);
        assert_eq!(r(f64::NAN, 2), 0.0);
        assert_eq!(opt(Some(f64::INFINITY), 1), None);
    }
}
