//! Poll LMU's read-only repair screen away from the high-frequency telemetry loop.
//! Dent severity in shared memory is not the aero/suspension state shown by the MFD.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::engine::Shared;
use crate::model::{Damage, Snapshot, Source};

const URL: &str = "http://127.0.0.1:6397/rest/garage/UIScreen/RepairAndRefuel";
const MAX_AGE: Duration = Duration::from_secs(3);

#[derive(Clone, PartialEq)]
struct Target {
    id: i32,
    vehicle: String,
    track: String,
    session: i32,
}

impl Target {
    fn from_snapshot(s: &Snapshot) -> Option<(Self, f64)> {
        if s.source != Source::Lmu || !s.connected {
            return None;
        }
        let p = s.player.as_ref()?;
        Some((
            Self {
                id: p.id,
                vehicle: p.vehicle_name.clone(),
                track: s.session.track.clone(),
                session: s.session.code,
            },
            p.elapsed_time,
        ))
    }
}

pub struct Reading {
    target: Target,
    elapsed: f64,
    received: Instant,
    damage: Damage,
}

impl Reading {
    pub fn apply(&self, s: &mut Snapshot) {
        let Some((target, elapsed)) = Target::from_snapshot(s) else {
            return;
        };
        if target == self.target && elapsed >= self.elapsed && self.received.elapsed() < MAX_AGE {
            if let Some(p) = s.player.as_mut() {
                p.damage = Some(self.damage.clone());
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepairScreen {
    team_info: TeamInfo,
    wearables: Wearables,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TeamInfo {
    vehicle_name: String,
}

#[derive(Deserialize)]
struct Wearables {
    body: Body,
    suspension: [f64; 4],
}

#[derive(Deserialize)]
struct Body {
    aero: f64,
}

fn parse(text: &str, vehicle: &str) -> Option<Damage> {
    let screen: RepairScreen = serde_json::from_str(text).ok()?;
    if screen.team_info.vehicle_name != vehicle {
        return None;
    }
    let w = screen.wearables;
    if !std::iter::once(&w.body.aero)
        .chain(w.suspension.iter())
        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    {
        return None;
    }
    Some(Damage {
        aero: w.body.aero,
        suspension: w.suspension,
    })
}

pub fn spawn(shared: Arc<Shared>) {
    thread::Builder::new()
        .name("damage".into())
        .spawn(move || {
            if rustls::crypto::CryptoProvider::get_default().is_none() {
                let _ = rustls::crypto::ring::default_provider().install_default();
            }
            // Loopback only. Do not follow redirects or use system proxies.
            let Ok(client) = reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_millis(750))
                .build()
            else {
                return;
            };
            loop {
                let target = shared
                    .latest
                    .read()
                    .ok()
                    .and_then(|s| Target::from_snapshot(&s));
                let reading = target.and_then(|(target, elapsed)| {
                    let text = tauri::async_runtime::block_on(async {
                        client
                            .get(URL)
                            .send()
                            .await
                            .ok()?
                            .error_for_status()
                            .ok()?
                            .text()
                            .await
                            .ok()
                    })?;
                    let damage = parse(&text, &target.vehicle)?;
                    Some(Reading {
                        target,
                        elapsed,
                        received: Instant::now(),
                        damage,
                    })
                });
                if let Ok(mut slot) = shared.damage.write() {
                    *slot = reading;
                }
                thread::sleep(Duration::from_millis(500));
            }
        })
        .expect("spawn damage reader");
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPAIR: &str = r#"{"teamInfo":{"vehicleName":"Test car"},"wearables":{"body":{"aero":0.0},"suspension":[0.0,0.12,0.0,0.0],"brakes":[0.034,0.034,0.032,0.032]}}"#;

    #[test]
    fn reads_repair_damage_without_inverting_healthy_zero() {
        let d = parse(REPAIR, "Test car").unwrap();
        assert_eq!(d.aero, 0.0);
        assert_eq!(d.suspension, [0.0, 0.12, 0.0, 0.0]);
    }

    #[test]
    fn rejects_other_cars_incomplete_arrays_and_invalid_fractions() {
        assert!(parse(REPAIR, "Other car").is_none());
        assert!(parse(&REPAIR.replace("0.12", "-1.0"), "Test car").is_none());
        assert!(parse(&REPAIR.replace("0.12", "1.01"), "Test car").is_none());
        assert!(parse(&REPAIR.replace("[0.0,0.12,0.0,0.0]", "[0.0]"), "Test car").is_none());
        assert!(parse("{}", "Test car").is_none());
    }

    #[test]
    fn does_not_apply_stale_or_mismatched_readings() {
        let mut s = crate::demo::DemoSim::new().step(0.02);
        s.source = Source::Lmu;
        s.player.as_mut().unwrap().damage = None;
        let (target, elapsed) = Target::from_snapshot(&s).unwrap();
        let mut r = Reading {
            target,
            elapsed,
            received: Instant::now(),
            damage: parse(REPAIR, "Test car").unwrap(),
        };
        r.apply(&mut s);
        assert!(s.player.as_ref().unwrap().damage.is_some());
        s.player.as_mut().unwrap().damage = None;
        r.received = Instant::now() - MAX_AGE;
        r.apply(&mut s);
        assert!(s.player.as_ref().unwrap().damage.is_none());
        r.received = Instant::now();
        r.elapsed = elapsed + 10.0;
        r.apply(&mut s);
        assert!(s.player.as_ref().unwrap().damage.is_none());
        r.elapsed = elapsed;
        r.target.id += 1;
        r.apply(&mut s);
        assert!(s.player.as_ref().unwrap().damage.is_none());
    }
}
