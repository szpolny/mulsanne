//! Synthetic data source: a car lapping a generated circuit with a multi-class
//! field. Used for layout editing and development without the game.

use std::f64::consts::TAU;

use crate::lmu::assign_class_places;
use crate::model::{Player, Session, SessionKind, Snapshot, Source, Vehicle, Weather, Wheel};

const TRACK_POINTS: usize = 2400;
const MAX_LAT_ACCEL: f64 = 22.0;
const MAX_ACCEL: f64 = 7.0;
const MAX_DECEL: f64 = 28.0;
const SESSION_LENGTH: f64 = 60.0 * 60.0;

struct Track {
    x: Vec<f64>,
    z: Vec<f64>,
    dist: Vec<f64>,
    curvature: Vec<f64>,
    speed: Vec<f64>,
    length: f64,
}

impl Track {
    fn generate() -> Self {
        let mut x = Vec::with_capacity(TRACK_POINTS);
        let mut z = Vec::with_capacity(TRACK_POINTS);
        for i in 0..TRACK_POINTS {
            let a = i as f64 / TRACK_POINTS as f64 * TAU;
            let r = 620.0 * (1.0 + 0.22 * (3.0 * a).sin() + 0.12 * (5.0 * a + 0.8).cos() + 0.05 * (9.0 * a).sin());
            x.push(r * a.cos() * 1.25);
            z.push(r * a.sin());
        }
        let n = TRACK_POINTS;
        let mut dist = vec![0.0; n];
        for i in 1..n {
            dist[i] = dist[i - 1] + (x[i] - x[i - 1]).hypot(z[i] - z[i - 1]);
        }
        let length = dist[n - 1] + (x[0] - x[n - 1]).hypot(z[0] - z[n - 1]);

        // Signed curvature from the turning angle between neighbouring segments.
        let mut curvature = vec![0.0; n];
        for i in 0..n {
            let (p, c, q) = ((i + n - 3) % n, i, (i + 3) % n);
            let a1 = (z[c] - z[p]).atan2(x[c] - x[p]);
            let a2 = (z[q] - z[c]).atan2(x[q] - x[c]);
            let mut da = a2 - a1;
            while da > std::f64::consts::PI {
                da -= TAU;
            }
            while da < -std::f64::consts::PI {
                da += TAU;
            }
            let ds = (x[q] - x[p]).hypot(z[q] - z[p]) / 2.0;
            curvature[i] = da / ds.max(1e-3);
        }

        // Grip-limited speed, then acceleration/braking limits both ways around the loop.
        let mut speed: Vec<f64> = curvature.iter().map(|k| (MAX_LAT_ACCEL / k.abs().max(1e-4)).sqrt().min(88.0)).collect();
        for _ in 0..2 {
            for i in 0..n {
                let j = (i + 1) % n;
                let ds = if j == 0 { length - dist[i] } else { dist[j] - dist[i] };
                speed[j] = speed[j].min((speed[i] * speed[i] + 2.0 * MAX_ACCEL * ds).sqrt());
            }
            for i in (0..n).rev() {
                let j = (i + 1) % n;
                let ds = if j == 0 { length - dist[i] } else { dist[j] - dist[i] };
                speed[i] = speed[i].min((speed[j] * speed[j] + 2.0 * MAX_DECEL * ds).sqrt());
            }
        }

        Self { x, z, dist, curvature, speed, length }
    }

    fn index_at(&self, d: f64) -> usize {
        let d = d.rem_euclid(self.length);
        self.dist.partition_point(|&x| x <= d).saturating_sub(1)
    }

    fn sample(&self, d: f64) -> (f64, f64, f64, f64) {
        let i = self.index_at(d);
        (self.x[i], self.z[i], self.speed[i], self.curvature[i])
    }
}

struct Opponent {
    id: i32,
    driver: String,
    vehicle: String,
    class: &'static str,
    pace: f64,
    progress: f64,
    laps: i32,
    best: f64,
    last: f64,
    lap_start: f64,
    pitstops: i16,
}

pub struct DemoSim {
    track: Track,
    time: f64,
    progress: f64,
    lap_start: f64,
    lap_number: i32,
    lap_variation: f64,
    fuel: f64,
    ve: f64,
    wear: f64,
    best: f64,
    last: f64,
    prev_speed: f64,
    invalid: bool,
    opponents: Vec<Opponent>,
    seed: u64,
}

impl DemoSim {
    pub fn new() -> Self {
        let classes = [("Hypercar", 1.0), ("LMP2", 0.93), ("LMP3", 0.88), ("LMGT3", 0.85)];
        let names = [
            "A. Laurent", "M. Rossi", "K. Tanaka", "J. Smith", "L. Novak", "P. Silva", "D. Müller", "S. Kowalski",
            "R. Jensen", "T. Dubois", "E. García", "N. Ivanov", "C. Brown", "F. Moreau", "H. Schmidt", "Y. Sato",
            "O. Nilsen", "G. Ricci", "B. Wilson", "V. Petrov",
        ];
        let cars = [
            ["Toyota GR010", "Porsche 963", "Ferrari 499P", "Cadillac V-Series.R", "BMW M Hybrid V8", "Alpine A424"],
            ["Oreca 07", "Oreca 07", "Oreca 07", "Oreca 07", "Oreca 07", "Oreca 07"],
            ["Ligier JS P325", "Duqueine D09", "Ginetta G61-LT-P3 Evo", "ADESS AD25", "Ligier JS P325", "Duqueine D09"],
            ["Porsche 911 GT3 R", "Ferrari 296 GT3", "BMW M4 GT3", "Corvette Z06 GT3.R", "Aston Martin Vantage", "Lexus RC F GT3"],
        ];
        let track = Track::generate();
        let mut opponents = Vec::new();
        for (i, name) in names.iter().enumerate() {
            let class_idx = i * classes.len() / names.len();
            let (class, pace) = classes[class_idx];
            let pace = pace * (0.985 + (i as f64 * 0.37).sin().abs() * 0.025);
            opponents.push(Opponent {
                id: i as i32 + 2,
                driver: name.to_string(),
                vehicle: format!("#{} {}", 10 + i * 3, cars[class_idx][i % 6]),
                class,
                pace,
                progress: track.length * (1.0 - i as f64 / names.len() as f64) * 0.97,
                laps: 0,
                best: 0.0,
                last: 0.0,
                lap_start: 0.0,
                pitstops: 0,
            });
        }
        Self {
            track,
            time: 0.0,
            progress: 0.0,
            lap_start: 0.0,
            lap_number: 0,
            lap_variation: 1.0,
            fuel: 70.0,
            ve: 1.0,
            wear: 1.0,
            best: 0.0,
            last: 0.0,
            prev_speed: 0.0,
            invalid: false,
            opponents,
            seed: 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn rand(&mut self) -> f64 {
        // xorshift64*
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;
        (self.seed.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn step(&mut self, dt: f64) -> Snapshot {
        self.time += dt;
        let len = self.track.length;

        // Player
        let (x, z, target, curvature) = self.track.sample(self.progress);
        let speed = target * self.lap_variation;
        self.progress += speed * dt;
        let accel = (speed - self.prev_speed) / dt.max(1e-3);
        self.prev_speed = speed;
        let throttle = if accel > 0.5 { 1.0 } else if accel > -0.5 { 0.55 + 0.3 * (speed / 88.0) } else { 0.0 };
        let brake = if accel < -2.0 { (-accel / MAX_DECEL).clamp(0.05, 1.0) } else { 0.0 };
        self.fuel = (self.fuel - throttle * dt * 0.034).max(0.0);
        self.ve = (self.ve - throttle * dt * 0.0005).max(0.0);
        self.wear = (self.wear - dt * 0.00002).max(0.0);

        if self.progress >= len {
            self.progress -= len;
            self.last = self.time - self.lap_start;
            if !self.invalid && (self.best == 0.0 || self.last < self.best) {
                self.best = self.last;
            }
            self.lap_start = self.time;
            self.lap_number += 1;
            self.lap_variation = 0.985 + self.rand() * 0.03;
            self.invalid = self.rand() < 0.15;
            if self.fuel < 8.0 {
                self.fuel = 70.0;
                self.ve = 1.0;
            }
        }

        let gear = ((speed / 88.0) * 7.0).ceil().clamp(1.0, 7.0) as i32;
        let gear_low = (gear - 1) as f64 * 88.0 / 7.0;
        let rpm = 4500.0 + (speed - gear_low) / (88.0 / 7.0) * 4000.0;
        let steering = (curvature * 18.0).clamp(-1.0, 1.0);

        let tyre = |base: f64, offset: f64| -> Wheel {
            let t = base + offset + (self.time * 0.3 + offset).sin() * 3.0;
            Wheel {
                surface_temp: [t + 4.0, t, t - 3.0],
                inner_temp: [t + 6.0, t + 2.0, t - 1.0],
                carcass_temp: t - 5.0,
                optimal_temp: 90.0,
                pressure: 168.0 + offset * 0.2,
                wear: self.wear - offset * 0.002,
                brake_temp: 350.0 + brake * 400.0 + offset,
                ride_height: 0.05,
                ..Default::default()
            }
        };

        let player = Player {
            id: 1,
            vehicle_name: "#99 Demo Racing".into(),
            vehicle_model: "Porsche 963".into(),
            vehicle_class: "Hypercar".into(),
            elapsed_time: self.time,
            lap_number: self.lap_number,
            lap_start_et: self.lap_start,
            lap_dist: self.progress,
            sector: if self.progress < len / 3.0 { 1 } else if self.progress < len * 2.0 / 3.0 { 2 } else { 3 },
            in_pits: false,
            speed,
            gear,
            max_gears: 7,
            rpm,
            max_rpm: 8800.0,
            throttle,
            brake,
            clutch: 0.0,
            steering,
            raw_throttle: throttle,
            raw_brake: brake,
            raw_steering: steering,
            steering_range_deg: 540.0,
            g_lat: speed * speed * curvature / 9.81,
            g_long: accel / 9.81,
            pos: [x, 0.0, z],
            yaw: 0.0,
            fuel: self.fuel,
            fuel_capacity: 75.0,
            virtual_energy: self.ve,
            battery: 0.5 + 0.3 * (self.time * 0.2).sin(),
            regen_kw: brake * 200.0,
            electric_motor_state: if brake > 0.0 { 3 } else { 2 },
            water_temp: 88.0,
            oil_temp: 102.0,
            turbo_boost: 0.0,
            delta_best: if self.best > 0.0 { (self.lap_variation - 1.0) * 40.0 * self.progress / len } else { 0.0 },
            lap_invalidated: self.invalid && self.progress > len * 0.4,
            track_limits_steps: 0,
            track_limits_steps_per_penalty: 20,
            track_limits_steps_per_point: 4,
            lift_and_coast: 0.0,
            tc: 4,
            tc_max: 11,
            tc_slip: 3,
            tc_cut: 2,
            abs: 0,
            abs_max: 0,
            tc_active: throttle > 0.9 && speed < 40.0,
            abs_active: false,
            motor_map: 2,
            motor_map_max: 5,
            migration: 3,
            front_arb: 3,
            rear_arb: 2,
            brake_bias_front: 0.565,
            speed_limiter: false,
            headlights: false,
            wipers: 0,
            overheating: false,
            detached: false,
            dents: [0, 0, 1, 0, 0, 0, 0, 2],
            last_impact_magnitude: 0.0,
            last_impact_et: 0.0,
            gap_ahead: 1.8,
            gap_behind: 2.4,
            front_compound: "Medium".into(),
            rear_compound: "Medium".into(),
            wheels: [tyre(82.0, 0.0), tyre(82.0, 4.0), tyre(85.0, 2.0), tyre(85.0, 6.0)],
        };

        // Opponents
        for o in &mut self.opponents {
            let (_, _, target, _) = self.track.sample(o.progress);
            o.progress += target * o.pace * dt;
            let laps = (o.progress / len).floor() as i32;
            if laps > o.laps {
                // The first crossing ends a partial lap, so no time for it.
                if o.laps > 0 {
                    let lap = self.time - o.lap_start;
                    o.last = lap;
                    if o.best == 0.0 || lap < o.best {
                        o.best = lap;
                    }
                }
                o.laps = laps;
                o.lap_start = self.time;
            }
        }

        let player_total = self.lap_number as f64 * len + self.progress;
        let mut vehicles: Vec<Vehicle> = Vec::with_capacity(self.opponents.len() + 1);
        vehicles.push(Vehicle {
            id: 1,
            driver: "You".into(),
            vehicle: player.vehicle_name.clone(),
            team: "Demo Racing".into(),
            class: "Hypercar".into(),
            is_player: true,
            total_laps: self.lap_number as i16,
            lap_dist: self.progress,
            sector: player.sector as u8,
            best_lap: self.best,
            last_lap: self.last,
            time_into_lap: self.time - self.lap_start,
            estimated_lap_time: if self.best > 0.0 { self.best } else { 110.0 },
            speed,
            pos: [x, 0.0, z],
            fuel_fraction: self.fuel / 75.0,
            number: "99".into(),
            model: "Porsche 963".into(),
            tyre_compound: "Medium".into(),
            tyre_wear: Some(self.wear),
            virtual_energy: Some(self.ve),
            ..Default::default()
        });
        for o in &self.opponents {
            let (ox, oz, target, _) = self.track.sample(o.progress);
            let lap_dist = o.progress.rem_euclid(len);
            vehicles.push(Vehicle {
                id: o.id,
                driver: o.driver.clone(),
                vehicle: o.vehicle.clone(),
                team: o.vehicle.split_once(' ').map(|(_, t)| t.to_string()).unwrap_or_default(),
                class: o.class.into(),
                total_laps: (o.progress / len).floor() as i16,
                lap_dist,
                sector: if lap_dist < len / 3.0 { 1 } else if lap_dist < len * 2.0 / 3.0 { 2 } else { 3 },
                best_lap: o.best,
                last_lap: o.last,
                time_into_lap: self.time - o.lap_start,
                estimated_lap_time: 110.0 / o.pace,
                num_pitstops: o.pitstops,
                speed: target * o.pace,
                pos: [ox, 0.0, oz],
                fuel_fraction: 0.6,
                number: crate::lmu::car_number(&o.vehicle),
                model: o.vehicle.split_once(' ').map(|(_, m)| m.to_string()).unwrap_or_default(),
                tyre_compound: ["Soft", "Medium", "Hard"][o.id as usize % 3].into(),
                tyre_wear: Some((1.0 - self.time * 0.00003 * (1.0 + (o.id % 4) as f64 * 0.3)).max(0.0)),
                // ~8% per lap, refilled when it runs out.
                virtual_energy: Some(1.0 - (o.progress / len * 0.08 + (o.id % 7) as f64 * 0.11).rem_euclid(0.95)),
                ..Default::default()
            });
        }

        // Order by total distance and derive gaps from an average speed.
        let totals: Vec<f64> = vehicles
            .iter()
            .map(|v| if v.is_player { player_total } else { v.total_laps as f64 * len + v.lap_dist })
            .collect();
        let mut order: Vec<usize> = (0..vehicles.len()).collect();
        order.sort_by(|&a, &b| totals[b].total_cmp(&totals[a]));
        let avg_speed = len / 110.0;
        let leader_total = totals[order[0]];
        for (place, &i) in order.iter().enumerate() {
            let v = &mut vehicles[i];
            v.place = place as u8 + 1;
            let behind = leader_total - totals[i];
            v.laps_behind_leader = (behind / len).floor() as i32;
            v.time_behind_leader = behind / avg_speed;
            if place > 0 {
                let prev = totals[order[place - 1]];
                v.time_behind_next = (prev - totals[i]) / avg_speed;
                v.laps_behind_next = ((prev - totals[i]) / len).floor() as i32;
            }
        }
        assign_class_places(&mut vehicles);

        let remaining = (SESSION_LENGTH - self.time).max(0.0);
        Snapshot {
            source: Source::Demo,
            connected: true,
            in_realtime: true,
            frame: 0,
            game_version: 0,
            session: Session {
                track: "Demo Circuit".into(),
                track_length: len,
                code: 1,
                kind: SessionKind::Practice,
                phase: 5,
                current_et: self.time,
                end_et: SESSION_LENGTH,
                time_remaining: remaining,
                max_laps: i32::MAX,
                time_of_day: 14.0 * 3600.0 + self.time,
                yellow_flag_state: 0,
                sector_yellow: [false; 3],
                sector_flags_raw: [11; 3],
                start_light: 0,
                num_red_lights: 0,
                player_name: "Demo Driver".into(),
                weather: Weather {
                    ambient_temp: 24.0,
                    track_temp: 36.0,
                    raining: 0.0,
                    dark_cloud: 0.2,
                    cloud_coverage: 2,
                    wind_speed: 3.5,
                    wind_direction: 0.8,
                    min_wetness: 0.0,
                    max_wetness: 0.0,
                    avg_wetness: 0.0,
                    grip_level: 3,
                },
            },
            player: Some(player),
            vehicles,
            timing: Default::default(),
            strategy: Default::default(),
        }
    }
}

