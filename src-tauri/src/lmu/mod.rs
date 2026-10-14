//! Native Le Mans Ultimate shared memory source.

pub mod layout;
pub mod shm;

use std::collections::HashMap;

use layout::{cstr, read_at, ScoringInfo, TelemInfo, VehicleScoring};

use crate::model::{Player, Session, SessionKind, Snapshot, Source, Vehicle, Weather, Wheel};

const KELVIN: f64 = 273.15;
const G: f64 = 9.80665;

fn mag(v: layout::Vec3) -> f64 {
    let (x, y, z) = (v.x, v.y, v.z);
    (x * x + y * y + z * z).sqrt()
}

/// Converts a copied `LMU_Data` block into a [`Snapshot`]. Timing and
/// strategy are filled in later by the recorder.
pub fn parse(buf: &[u8]) -> Snapshot {
    let game_version: i32 = read_at(buf, layout::OFF_GENERIC_GAME_VERSION);
    let info: ScoringInfo = read_at(buf, layout::OFF_SCORING_INFO);

    let num_vehicles = (info.num_vehicles.max(0) as usize).min(layout::MAX_VEHICLES);
    let mut vehicles: Vec<Vehicle> = (0..num_vehicles)
        .map(|i| {
            let v: VehicleScoring = read_at(
                buf,
                layout::OFF_SCORING_VEHICLES + i * std::mem::size_of::<VehicleScoring>(),
            );
            parse_vehicle(&v)
        })
        .collect();
    assign_class_places(&mut vehicles);

    let session = parse_session(&info);

    let active: u8 = read_at(buf, layout::OFF_TELEMETRY_ACTIVE_VEHICLES);
    // Telemetry is available for every car, not just the player: use it for tyre and energy info.
    let telemetry: HashMap<i32, TelemInfo> = (0..(active as usize).min(layout::MAX_VEHICLES))
        .map(|i| {
            let t: TelemInfo = read_at(buf, layout::OFF_TELEMETRY_INFO + i * std::mem::size_of::<TelemInfo>());
            (t.id, t)
        })
        .collect();
    for v in &mut vehicles {
        if let Some(t) = telemetry.get(&v.id) {
            v.model = cstr(&t.vehicle_model);
            v.tyre_compound = cstr(&t.front_tire_compound_name);
            let wheels = t.wheels;
            let wear = wheels.iter().map(|w| { w.wear }).sum::<f64>() / 4.0;
            // Remote cars may report 0 when the server does not share wear.
            v.tyre_wear = (wear > 0.0 && wear <= 1.0).then_some(wear);
            // 0 for cars without virtual energy (or when not shared); the value may be 0..1 or 0..100.
            let ve = t.virtual_energy as f64;
            v.virtual_energy = (ve > 0.0).then(|| if ve > 1.0001 { ve / 100.0 } else { ve });
        }
    }
    let player_idx: u8 = read_at(buf, layout::OFF_TELEMETRY_PLAYER_IDX);
    let has_player: u8 = read_at(buf, layout::OFF_TELEMETRY_PLAYER_HAS_VEHICLE);

    let player = if has_player != 0 && (player_idx as usize) < (active as usize).min(layout::MAX_VEHICLES) {
        let t: TelemInfo = read_at(
            buf,
            layout::OFF_TELEMETRY_INFO + player_idx as usize * std::mem::size_of::<TelemInfo>(),
        );
        let scoring = vehicles.iter().find(|v| v.is_player).or_else(|| vehicles.iter().find(|v| v.id == t.id));
        let mut player = parse_player(&t, scoring, &info);
        player.ffb = read_at::<f32>(buf, layout::OFF_GENERIC_FFB_TORQUE) as f64;
        Some(player)
    } else {
        None
    };

    Snapshot {
        source: Source::Lmu,
        connected: true,
        in_realtime: info.in_realtime != 0,
        frame: 0,
        game_version,
        session,
        player,
        vehicles,
        timing: Default::default(),
        strategy: Default::default(),
    }
}

fn parse_session(info: &ScoringInfo) -> Session {
    let wind = info.wind;
    let (wx, wz) = (wind.x, wind.z);
    let remaining = if info.end_et > 0.0 { (info.end_et - info.current_et).max(0.0) } else { 0.0 };
    Session {
        track: cstr(&info.track_name),
        track_length: info.lap_dist,
        code: info.session,
        kind: SessionKind::from_code(info.session),
        phase: info.game_phase,
        current_et: info.current_et,
        end_et: info.end_et,
        time_remaining: if info.session_time_remaining > 0.0 { info.session_time_remaining as f64 } else { remaining },
        max_laps: info.max_laps,
        time_of_day: info.time_of_day as f64,
        yellow_flag_state: info.yellow_flag_state,
        sector_yellow: sector_yellow(info.sector_flag),
        sector_flags_raw: info.sector_flag,
        start_light: info.start_light,
        num_red_lights: info.num_red_lights,
        player_name: cstr(&info.player_name),
        weather: Weather {
            ambient_temp: info.ambient_temp,
            track_temp: info.track_temp,
            raining: info.raining,
            dark_cloud: info.dark_cloud,
            cloud_coverage: info.cloud_coverage,
            wind_speed: (wx * wx + wz * wz).sqrt(),
            wind_direction: wx.atan2(wz),
            min_wetness: info.min_path_wetness,
            max_wetness: info.max_path_wetness,
            avg_wetness: info.avg_path_wetness,
            grip_level: info.track_grip_level,
        },
    }
}

/// The scoring array is indexed like `mSector` (0 = S3, 1 = S1, 2 = S2).
/// LMU reports 11 for a green sector and 1 (rF2 convention) for a local yellow.
fn sector_yellow(raw: [i8; 3]) -> [bool; 3] {
    let yellow = |v: i8| v == 1;
    [yellow(raw[1]), yellow(raw[2]), yellow(raw[0])]
}

fn parse_vehicle(v: &VehicleScoring) -> Vehicle {
    let pos = v.pos;
    Vehicle {
        id: v.id,
        place: v.place,
        class_place: 0,
        driver: cstr(&v.driver_name),
        vehicle: cstr(&v.vehicle_name),
        team: cstr(&v.pit_group),
        class: cstr(&v.vehicle_class),
        is_player: v.is_player != 0,
        total_laps: v.total_laps,
        lap_dist: v.lap_dist,
        sector: match v.sector {
            1 => 1,
            2 => 2,
            _ => 3,
        },
        best_lap: v.best_lap_time,
        last_lap: v.last_lap_time,
        best_s1: v.best_sector1,
        best_s2: v.best_sector2,
        last_s1: v.last_sector1,
        last_s2: v.last_sector2,
        cur_s1: v.cur_sector1,
        cur_s2: v.cur_sector2,
        time_behind_next: v.time_behind_next,
        laps_behind_next: v.laps_behind_next,
        time_behind_leader: v.time_behind_leader,
        laps_behind_leader: v.laps_behind_leader,
        time_into_lap: v.time_into_lap,
        estimated_lap_time: v.estimated_lap_time,
        in_pits: v.in_pits != 0,
        pit_state: v.pit_state,
        in_garage: v.in_garage_stall != 0,
        num_pitstops: v.num_pitstops,
        num_penalties: v.num_penalties,
        finish_status: v.finish_status,
        flag: v.flag,
        fuel_fraction: v.fuel_fraction as f64 / 255.0,
        control: v.control,
        speed: mag(v.local_vel),
        pos: [pos.x, pos.y, pos.z],
        number: car_number(&cstr(&v.vehicle_name)),
        ..Default::default()
    }
}

/// "Toyota Racing 2026 #7:WEC" -> "7".
pub fn car_number(name: &str) -> String {
    name.split_once('#')
        .map(|(_, rest)| rest.chars().take_while(|c| c.is_ascii_alphanumeric()).collect())
        .unwrap_or_default()
}

/// Fills `class_place` (1-based position within the vehicle's class).
pub fn assign_class_places(vehicles: &mut [Vehicle]) {
    let mut order: Vec<usize> = (0..vehicles.len()).collect();
    order.sort_by_key(|&i| vehicles[i].place);
    let mut counters: HashMap<String, u16> = HashMap::new();
    for i in order {
        let c = counters.entry(vehicles[i].class.clone()).or_insert(0);
        *c += 1;
        vehicles[i].class_place = *c;
    }
}

fn parse_wheel(w: &layout::TelemWheel) -> Wheel {
    let t = w.temperature;
    let inner = w.tire_inner_layer_temperature;
    Wheel {
        surface_temp: [t[0] - KELVIN, t[1] - KELVIN, t[2] - KELVIN],
        inner_temp: [inner[0] - KELVIN, inner[1] - KELVIN, inner[2] - KELVIN],
        carcass_temp: w.tire_carcass_temperature - KELVIN,
        optimal_temp: {
            let o = w.optimal_temp as f64;
            // Reported in Kelvin by current builds; tolerate Celsius as well.
            if o > 200.0 { o - KELVIN } else { o }
        },
        pressure: w.pressure,
        wear: w.wear,
        brake_temp: w.brake_temp,
        ride_height: w.ride_height,
        flat: w.flat != 0,
        detached: w.detached != 0,
        surface_type: w.surface_type,
    }
}

fn parse_player(t: &TelemInfo, scoring: Option<&Vehicle>, info: &ScoringInfo) -> Player {
    let vel = t.local_vel;
    let acc = t.local_accel;
    let pos = t.pos;
    let ori_z = t.ori[2];
    let (oz_x, oz_z) = (ori_z.x, ori_z.z);
    let wheels = t.wheels;

    Player {
        id: t.id,
        vehicle_name: cstr(&t.vehicle_name),
        vehicle_model: cstr(&t.vehicle_model),
        vehicle_class: scoring.map(|s| s.class.clone()).unwrap_or_default(),
        elapsed_time: t.elapsed_time,
        lap_number: t.lap_number,
        lap_start_et: t.lap_start_et,
        lap_dist: scoring.map(|s| s.lap_dist).unwrap_or(0.0),
        sector: scoring.map(|s| s.sector as i8).unwrap_or(0),
        in_pits: scoring.map(|s| s.in_pits).unwrap_or(false) || t.current_sector < 0,

        speed: mag(vel),
        gear: t.gear,
        max_gears: t.max_gears,
        rpm: t.engine_rpm,
        max_rpm: t.engine_max_rpm,
        throttle: t.filtered_throttle,
        brake: t.filtered_brake,
        clutch: t.filtered_clutch,
        steering: t.filtered_steering,
        raw_throttle: t.unfiltered_throttle,
        raw_brake: t.unfiltered_brake,
        raw_steering: t.unfiltered_steering,
        steering_range_deg: t.physical_steering_wheel_range as f64,
        g_lat: acc.x / G,
        g_long: -acc.z / G,
        pos: [pos.x, pos.y, pos.z],
        yaw: oz_x.atan2(oz_z),

        fuel: t.fuel,
        fuel_capacity: t.fuel_capacity,
        virtual_energy: t.virtual_energy as f64,
        battery: t.battery_charge_fraction,
        regen_kw: t.regen as f64,
        electric_motor_state: t.electric_boost_motor_state,
        water_temp: t.engine_water_temp,
        oil_temp: t.engine_oil_temp,
        turbo_boost: t.turbo_boost_pressure,

        delta_best: t.delta_best,
        lap_invalidated: t.lap_invalidated != 0,
        track_limits_steps: t.track_limits_steps,
        track_limits_steps_per_penalty: info.track_limits_steps_per_penalty,
        track_limits_steps_per_point: info.track_limits_steps_per_point,
        lift_and_coast: t.lift_and_coast_progress as f64 / 255.0,

        tc: t.tc,
        tc_max: t.tc_max,
        tc_slip: t.tc_slip,
        tc_cut: t.tc_cut,
        abs: t.abs,
        abs_max: t.abs_max,
        tc_active: t.tc_active != 0,
        abs_active: t.abs_active != 0,
        motor_map: t.motor_map,
        motor_map_max: t.motor_map_max,
        migration: t.migration,
        front_arb: t.front_anti_sway,
        rear_arb: t.rear_anti_sway,
        brake_bias_front: 1.0 - t.rear_brake_bias,
        speed_limiter: t.speed_limiter_active != 0 || t.speed_limiter != 0,
        headlights: t.headlights != 0,
        wipers: t.wiper_state,
        ignition: t.ignition_starter,
        anti_stall: t.anti_stall_activated != 0,
        ffb: 0.0,

        overheating: t.overheating != 0,
        detached: t.detached != 0,
        dents: t.dent_severity,
        last_impact_magnitude: t.last_impact_magnitude,
        last_impact_et: t.last_impact_et,

        gap_ahead: t.time_gap_car_ahead as f64,
        gap_behind: t.time_gap_car_behind as f64,

        front_compound: cstr(&t.front_tire_compound_name),
        rear_compound: cstr(&t.rear_tire_compound_name),
        wheels: [
            parse_wheel(&wheels[0]),
            parse_wheel(&wheels[1]),
            parse_wheel(&wheels[2]),
            parse_wheel(&wheels[3]),
        ],
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_car_numbers() {
        assert_eq!(super::car_number("Toyota Racing 2026 #7:WEC"), "7");
        assert_eq!(super::car_number("Aston Martin THOR Team 2026 #007:WEC"), "007");
        assert_eq!(super::car_number("No number"), "");
    }
}
