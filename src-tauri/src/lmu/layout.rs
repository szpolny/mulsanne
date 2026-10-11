//! Binary layout of the native LMU shared memory block (`LMU_Data`).
//!
//! Mirrors the structures from `Support/SharedMemoryInterface` in the game
//! folder. The inner structs use `#pragma pack(4)`; the outer wrapper structs
//! use default packing, so they are addressed through the offsets below
//! instead of Rust structs. Sizes and offsets are checked in the tests at the
//! bottom against values measured with MSVC.

#![allow(dead_code)]

pub const MAPPING_NAME: &str = "LMU_Data";
pub const HOLD_EVENT_NAME: &str = "LMU_Data_HoldEvent";
pub const DATA_EVENT_NAME: &str = "LMU_Data_DataEvent";
pub const LOCK_MAPPING_NAME: &str = "LMU_SharedMemoryLockData";
pub const LOCK_EVENT_NAME: &str = "LMU_SharedMemoryLockEvent";

pub const MAX_VEHICLES: usize = 104;
pub const SME_MAX: usize = 16;

// Offsets inside SharedMemoryObjectOut.
pub const OFF_GENERIC_EVENTS: usize = 0;
pub const OFF_GENERIC_GAME_VERSION: usize = 64;
pub const OFF_GENERIC_FFB_TORQUE: usize = 68;
pub const OFF_GENERIC_APP_INFO: usize = 72;
pub const OFF_PATHS: usize = 332;
pub const OFF_SCORING: usize = 1632;
pub const OFF_SCORING_INFO: usize = OFF_SCORING;
pub const OFF_SCORING_VEHICLES: usize = OFF_SCORING + 560;
pub const OFF_TELEMETRY: usize = 128_464;
pub const OFF_TELEMETRY_ACTIVE_VEHICLES: usize = OFF_TELEMETRY;
pub const OFF_TELEMETRY_PLAYER_IDX: usize = OFF_TELEMETRY + 1;
pub const OFF_TELEMETRY_PLAYER_HAS_VEHICLE: usize = OFF_TELEMETRY + 2;
pub const OFF_TELEMETRY_INFO: usize = OFF_TELEMETRY + 4;
pub const TOTAL_SIZE: usize = 324_824;

#[repr(C, packed(4))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct TelemWheel {
    pub suspension_deflection: f64,
    pub ride_height: f64,
    pub susp_force: f64,
    pub brake_temp: f64, // Celsius
    pub brake_pressure: f64,
    pub rotation: f64,
    pub lateral_patch_vel: f64,
    pub longitudinal_patch_vel: f64,
    pub lateral_ground_vel: f64,
    pub longitudinal_ground_vel: f64,
    pub camber: f64,
    pub lateral_force: f64,
    pub longitudinal_force: f64,
    pub tire_load: f64,
    pub grip_fract: f64,
    pub pressure: f64,         // kPa
    pub temperature: [f64; 3], // Kelvin, left/center/right
    pub wear: f64,
    pub terrain_name: [u8; 16],
    pub surface_type: u8,
    pub flat: u8,
    pub detached: u8,
    pub static_undeflected_radius: u8,
    pub vertical_tire_deflection: f64,
    pub wheel_y_location: f64,
    pub toe: f64,
    pub tire_carcass_temperature: f64,
    pub tire_inner_layer_temperature: [f64; 3],
    pub optimal_temp: f32,
    pub compound_index: u8,
    pub compound_type: u8,
    pub expansion: [u8; 18],
}

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct TelemInfo {
    pub id: i32,
    pub delta_time: f64,
    pub elapsed_time: f64,
    pub lap_number: i32,
    pub lap_start_et: f64,
    pub vehicle_name: [u8; 64],
    pub track_name: [u8; 64],

    pub pos: Vec3,
    pub local_vel: Vec3,
    pub local_accel: Vec3,
    pub ori: [Vec3; 3],
    pub local_rot: Vec3,
    pub local_rot_accel: Vec3,

    pub gear: i32,
    pub engine_rpm: f64,
    pub engine_water_temp: f64,
    pub engine_oil_temp: f64,
    pub clutch_rpm: f64,

    pub unfiltered_throttle: f64,
    pub unfiltered_brake: f64,
    pub unfiltered_steering: f64,
    pub unfiltered_clutch: f64,
    pub filtered_throttle: f64,
    pub filtered_brake: f64,
    pub filtered_steering: f64,
    pub filtered_clutch: f64,

    pub steering_shaft_torque: f64,
    pub front_3rd_deflection: f64,
    pub rear_3rd_deflection: f64,

    pub front_wing_height: f64,
    pub front_ride_height: f64,
    pub rear_ride_height: f64,
    pub drag: f64,
    pub front_downforce: f64,
    pub rear_downforce: f64,

    pub fuel: f64,
    pub engine_max_rpm: f64,
    pub scheduled_stops: u8,
    pub overheating: u8,
    pub detached: u8,
    pub headlights: u8,
    pub dent_severity: [u8; 8],
    pub last_impact_et: f64,
    pub last_impact_magnitude: f64,
    pub last_impact_pos: Vec3,

    pub engine_torque: f64,
    pub current_sector: i32,
    pub speed_limiter: u8,
    pub max_gears: u8,
    pub front_tire_compound_index: u8,
    pub rear_tire_compound_index: u8,
    pub fuel_capacity: f64,
    pub front_flap_activated: u8,
    pub rear_flap_activated: u8,
    pub rear_flap_legal_status: u8,
    pub ignition_starter: u8,
    pub front_tire_compound_name: [u8; 18],
    pub rear_tire_compound_name: [u8; 18],
    pub speed_limiter_available: u8,
    pub anti_stall_activated: u8,
    pub unused: [u8; 2],
    pub visual_steering_wheel_range: f32,
    pub rear_brake_bias: f64,
    pub turbo_boost_pressure: f64,
    pub physics_to_graphics_offset: [f32; 3],
    pub physical_steering_wheel_range: f32,

    pub delta_best: f64,
    pub battery_charge_fraction: f64,
    pub electric_boost_motor_torque: f64,
    pub electric_boost_motor_rpm: f64,
    pub electric_boost_motor_temperature: f64,
    pub electric_boost_water_temperature: f64,
    pub electric_boost_motor_state: u8,
    pub lap_invalidated: u8,
    pub abs_active: u8,
    pub tc_active: u8,
    pub speed_limiter_active: u8,
    pub wiper_state: u8,
    pub tc: u8,
    pub tc_max: u8,
    pub tc_slip: u8,
    pub tc_slip_max: u8,
    pub tc_cut: u8,
    pub tc_cut_max: u8,
    pub abs: u8,
    pub abs_max: u8,
    pub motor_map: u8,
    pub motor_map_max: u8,
    pub migration: u8,
    pub migration_max: u8,
    pub front_anti_sway: u8,
    pub front_anti_sway_max: u8,
    pub rear_anti_sway: u8,
    pub rear_anti_sway_max: u8,
    pub lift_and_coast_progress: u8,
    pub track_limits_steps: u8,
    pub regen: f32,
    pub soc: f32,
    pub virtual_energy: f32,
    pub time_gap_car_ahead: f32,
    pub time_gap_car_behind: f32,
    pub time_gap_place_ahead: f32,
    pub time_gap_place_behind: f32,
    pub vehicle_model: [u8; 30],
    pub vehicle_class: u8,
    pub vehicle_championship: u8,
    pub expansion: [u8; 20],

    pub wheels: [TelemWheel; 4], // FL, FR, RL, RR
}

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct VehicleScoring {
    pub id: i32,
    pub driver_name: [u8; 32],
    pub vehicle_name: [u8; 64],
    pub total_laps: i16,
    pub sector: i8, // 0=sector3, 1=sector1, 2=sector2
    pub finish_status: i8,
    pub lap_dist: f64,
    pub path_lateral: f64,
    pub track_edge: f64,

    pub best_sector1: f64,
    pub best_sector2: f64,
    pub best_lap_time: f64,
    pub last_sector1: f64,
    pub last_sector2: f64,
    pub last_lap_time: f64,
    pub cur_sector1: f64,
    pub cur_sector2: f64,

    pub num_pitstops: i16,
    pub num_penalties: i16,
    pub is_player: u8,
    pub control: i8,
    pub in_pits: u8,
    pub place: u8,
    pub vehicle_class: [u8; 32],

    pub time_behind_next: f64,
    pub laps_behind_next: i32,
    pub time_behind_leader: f64,
    pub laps_behind_leader: i32,
    pub lap_start_et: f64,

    pub pos: Vec3,
    pub local_vel: Vec3,
    pub local_accel: Vec3,
    pub ori: [Vec3; 3],
    pub local_rot: Vec3,
    pub local_rot_accel: Vec3,

    pub headlights: u8,
    pub pit_state: u8,
    pub server_scored: u8,
    pub individual_phase: u8,
    pub qualification: i32,
    pub time_into_lap: f64,
    pub estimated_lap_time: f64,
    pub pit_group: [u8; 24],
    pub flag: u8,
    pub under_yellow: u8,
    pub count_lap_flag: u8,
    pub in_garage_stall: u8,
    pub upgrade_pack: [u8; 16],
    pub pit_lap_dist: f32,
    pub best_lap_sector1: f32,
    pub best_lap_sector2: f32,
    pub steam_id: u64,
    pub veh_filename: [u8; 32],
    pub attack_mode: i16,
    pub fuel_fraction: u8,
    pub drs_state: u8,
    pub expansion: [u8; 4],
}

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct ScoringInfo {
    pub track_name: [u8; 64],
    pub session: i32,
    pub current_et: f64,
    pub end_et: f64,
    pub max_laps: i32,
    pub lap_dist: f64,
    pub results_stream_ptr: u64,
    pub num_vehicles: i32,
    pub game_phase: u8,
    pub yellow_flag_state: i8,
    pub sector_flag: [i8; 3],
    pub start_light: u8,
    pub num_red_lights: u8,
    pub in_realtime: u8,
    pub player_name: [u8; 32],
    pub plr_file_name: [u8; 64],
    pub dark_cloud: f64,
    pub raining: f64,
    pub ambient_temp: f64,
    pub track_temp: f64,
    pub wind: Vec3,
    pub min_path_wetness: f64,
    pub max_path_wetness: f64,
    pub game_mode: u8,
    pub is_password_protected: u8,
    pub server_port: u16,
    pub server_public_ip: u32,
    pub max_players: i32,
    pub server_name: [u8; 32],
    pub start_et: f32,
    pub avg_path_wetness: f64,
    pub session_time_remaining: f32,
    pub time_of_day: f32,
    pub is_fixed_setup: u8,
    pub track_grip_level: u8,
    pub cloud_coverage: u8,
    pub track_limits_steps_per_penalty: u8,
    pub track_limits_steps_per_point: u8,
    pub expansion: [u8; 187],
    pub vehicle_ptr: u64,
}

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct ApplicationState {
    pub app_window: u64, // HWND
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub windowed: u32,
    pub options_location: u8, // 0=main UI, 1=track loading, 2=monitor, 3=on track
    pub options_page: [u8; 31],
    pub expansion: [u8; 204],
}

/// Reads a `T` from `buf` at `offset`. `T` must be a plain-old-data struct
/// from this module (all bit patterns valid).
pub fn read_at<T: Copy>(buf: &[u8], offset: usize) -> T {
    assert!(offset + std::mem::size_of::<T>() <= buf.len());
    // SAFETY: bounds checked above; every type in this module is POD made of
    // integers, floats and byte arrays, so any bit pattern is a valid value.
    unsafe { std::ptr::read_unaligned(buf.as_ptr().add(offset) as *const T) }
}

/// Converts a fixed-size, NUL-terminated C string to a Rust string.
pub fn cstr(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    // Expected values were measured by compiling the official headers with MSVC x64.
    #[test]
    fn struct_sizes_match_sdk() {
        assert_eq!(size_of::<Vec3>(), 24);
        assert_eq!(size_of::<TelemWheel>(), 260);
        assert_eq!(size_of::<TelemInfo>(), 1888);
        assert_eq!(size_of::<VehicleScoring>(), 584);
        assert_eq!(size_of::<ScoringInfo>(), 548);
        assert_eq!(size_of::<ApplicationState>(), 260);
    }

    #[test]
    fn field_offsets_match_sdk() {
        assert_eq!(offset_of!(TelemWheel, optimal_temp), 236);
        assert_eq!(offset_of!(TelemWheel, expansion), 242);

        assert_eq!(offset_of!(TelemInfo, fuel_capacity), 608);
        assert_eq!(offset_of!(TelemInfo, rear_brake_bias), 664);
        assert_eq!(offset_of!(TelemInfo, delta_best), 696);
        assert_eq!(offset_of!(TelemInfo, regen), 768);
        assert_eq!(offset_of!(TelemInfo, vehicle_model), 796);
        assert_eq!(offset_of!(TelemInfo, expansion), 828);
        assert_eq!(offset_of!(TelemInfo, wheels), 848);

        assert_eq!(offset_of!(VehicleScoring, time_behind_next), 232);
        assert_eq!(offset_of!(VehicleScoring, pit_lap_dist), 524);
        assert_eq!(offset_of!(VehicleScoring, steam_id), 536);
        assert_eq!(offset_of!(VehicleScoring, fuel_fraction), 578);

        assert_eq!(offset_of!(ScoringInfo, results_stream_ptr), 96);
        assert_eq!(offset_of!(ScoringInfo, num_vehicles), 104);
        assert_eq!(offset_of!(ScoringInfo, server_public_ip), 288);
        assert_eq!(offset_of!(ScoringInfo, avg_path_wetness), 332);
        assert_eq!(offset_of!(ScoringInfo, session_time_remaining), 340);
        assert_eq!(offset_of!(ScoringInfo, vehicle_ptr), 540);
    }

    #[test]
    fn block_offsets_are_consistent() {
        assert_eq!(OFF_GENERIC_APP_INFO + size_of::<ApplicationState>(), OFF_PATHS);
        assert_eq!(OFF_SCORING_VEHICLES + MAX_VEHICLES * size_of::<VehicleScoring>() + 65536, OFF_TELEMETRY);
        assert_eq!(OFF_TELEMETRY_INFO + MAX_VEHICLES * size_of::<TelemInfo>() + 4, TOTAL_SIZE);
    }
}
