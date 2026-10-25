//! Game-agnostic snapshot sent to the overlay and companion windows.
//! Units: distances m, speeds m/s, temperatures °C, pressures kPa, times s.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    #[default]
    None,
    Lmu,
    Demo,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub source: Source,
    pub connected: bool,
    pub in_realtime: bool,
    /// Monotonic counter, increments for every new frame.
    pub frame: u64,
    pub game_version: i32,
    pub session: Session,
    pub player: Option<Player>,
    pub vehicles: Vec<Vehicle>,
    pub timing: Timing,
    pub strategy: Strategy,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub track: String,
    pub track_length: f64,
    /// Raw session code: 0 test day, 1-4 practice, 5-8 qualifying, 9 warmup, 10-13 race.
    pub code: i32,
    pub kind: SessionKind,
    /// 0 before start .. 5 green, 6 FCY/SC, 7 stopped, 8 over, 9 paused.
    pub phase: u8,
    pub current_et: f64,
    pub end_et: f64,
    pub time_remaining: f64,
    pub max_laps: i32,
    pub time_of_day: f64,
    pub yellow_flag_state: i8,
    /// Local yellow per sector, in track order S1, S2, S3.
    pub sector_yellow: [bool; 3],
    /// Raw per-sector flag values as reported by the game (for diagnostics).
    pub sector_flags_raw: [i8; 3],
    pub start_light: u8,
    pub num_red_lights: u8,
    pub player_name: String,
    pub weather: Weather,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionKind {
    #[default]
    Unknown,
    TestDay,
    Practice,
    Qualifying,
    Warmup,
    Race,
}

impl SessionKind {
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => Self::TestDay,
            1..=4 => Self::Practice,
            5..=8 => Self::Qualifying,
            9 => Self::Warmup,
            10..=13 => Self::Race,
            _ => Self::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::TestDay => "Test Day",
            Self::Practice => "Practice",
            Self::Qualifying => "Qualifying",
            Self::Warmup => "Warmup",
            Self::Race => "Race",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    pub ambient_temp: f64,
    pub track_temp: f64,
    pub raining: f64,
    pub dark_cloud: f64,
    pub cloud_coverage: u8,
    pub wind_speed: f64,
    /// Direction the wind blows towards, radians in world XZ plane.
    pub wind_direction: f64,
    pub min_wetness: f64,
    pub max_wetness: f64,
    pub avg_wetness: f64,
    pub grip_level: u8,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: i32,
    pub vehicle_name: String,
    pub vehicle_model: String,
    pub vehicle_class: String,
    pub elapsed_time: f64,
    pub lap_number: i32,
    pub lap_start_et: f64,
    pub lap_dist: f64,
    pub sector: i8,
    pub in_pits: bool,

    pub speed: f64,
    pub gear: i32,
    pub max_gears: u8,
    pub rpm: f64,
    pub max_rpm: f64,
    pub throttle: f64,
    pub brake: f64,
    pub clutch: f64,
    pub steering: f64,
    pub raw_throttle: f64,
    pub raw_brake: f64,
    pub raw_steering: f64,
    pub steering_range_deg: f64,
    pub g_lat: f64,
    pub g_long: f64,
    pub pos: [f64; 3],
    pub yaw: f64,

    pub fuel: f64,
    pub fuel_capacity: f64,
    pub virtual_energy: f64,
    pub battery: f64,
    pub regen_kw: f64,
    pub electric_motor_state: u8,
    pub water_temp: f64,
    pub oil_temp: f64,
    pub turbo_boost: f64,

    pub delta_best: f64,
    pub lap_invalidated: bool,
    pub track_limits_steps: u8,
    pub track_limits_steps_per_penalty: u8,
    pub track_limits_steps_per_point: u8,
    pub lift_and_coast: f64,

    pub tc: u8,
    pub tc_max: u8,
    pub tc_slip: u8,
    pub tc_cut: u8,
    pub abs: u8,
    pub abs_max: u8,
    pub tc_active: bool,
    pub abs_active: bool,
    pub motor_map: u8,
    pub motor_map_max: u8,
    pub migration: u8,
    pub front_arb: u8,
    pub rear_arb: u8,
    pub brake_bias_front: f64,
    pub speed_limiter: bool,
    pub headlights: bool,
    pub wipers: u8,
    /// 0 = off, 1 = ignition on, 2 = ignition + starter.
    pub ignition: u8,
    pub anti_stall: bool,
    /// Force feedback output, -1..1 (|1| = clipping).
    pub ffb: f64,

    pub overheating: bool,
    pub detached: bool,
    pub dents: [u8; 8],
    pub damage: Option<Damage>,
    pub last_impact_magnitude: f64,
    pub last_impact_et: f64,

    pub gap_ahead: f64,
    pub gap_behind: f64,

    pub front_compound: String,
    pub rear_compound: String,
    pub wheels: [Wheel; 4],
}

/// Repair-screen wearables. Fractions describe damage (0 = healthy).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Damage {
    pub aero: f64,
    /// FL, FR, RL, RR.
    pub suspension: [f64; 4],
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Wheel {
    /// Surface temperature °C (left / centre / right).
    pub surface_temp: [f64; 3],
    pub inner_temp: [f64; 3],
    pub carcass_temp: f64,
    pub optimal_temp: f64,
    pub pressure: f64,
    /// Remaining tread fraction 0..1 (1 = new).
    pub wear: f64,
    pub brake_temp: f64,
    pub ride_height: f64,
    pub flat: bool,
    pub detached: bool,
    pub surface_type: u8,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    pub id: i32,
    pub place: u8,
    pub class_place: u16,
    pub driver: String,
    pub vehicle: String,
    pub team: String,
    pub class: String,
    pub is_player: bool,
    pub total_laps: i16,
    pub lap_dist: f64,
    /// 1, 2 or 3.
    pub sector: u8,
    pub best_lap: f64,
    pub last_lap: f64,
    pub best_s1: f64,
    pub best_s2: f64,
    pub last_s1: f64,
    pub last_s2: f64,
    pub cur_s1: f64,
    pub cur_s2: f64,
    pub time_behind_next: f64,
    pub laps_behind_next: i32,
    pub time_behind_leader: f64,
    pub laps_behind_leader: i32,
    pub time_into_lap: f64,
    pub estimated_lap_time: f64,
    pub in_pits: bool,
    pub pit_state: u8,
    pub in_garage: bool,
    pub num_pitstops: i16,
    pub num_penalties: i16,
    /// Details of the unserved penalties, oldest first, read from the game's
    /// trace log. May hold fewer entries than `num_penalties` when the log
    /// has no matching line.
    pub penalties: Vec<Penalty>,
    pub finish_status: i8,
    pub flag: u8,
    pub fuel_fraction: f64,
    pub control: i8,
    pub speed: f64,
    pub pos: [f64; 3],
    /// Race number parsed from the entry name (e.g. "#007" -> "007").
    pub number: String,
    /// Car model, e.g. "Porsche 963" (empty when the game does not report it).
    pub model: String,
    /// Front tyre compound name, e.g. "Soft".
    pub tyre_compound: String,
    /// Average remaining tread 0..1 across the four tyres (1 = new), if known.
    pub tyre_wear: Option<f64>,
    /// Laps completed on the current set (since the last pit stop).
    pub tyre_laps: i32,
    /// Remaining virtual energy 0..1, if the car uses it and the game shares it.
    pub virtual_energy: Option<f64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PenaltyKind {
    StopGo,
    DriveThrough,
    #[default]
    Other,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Penalty {
    pub kind: PenaltyKind,
    /// Stationary time for a stop/go, 0 otherwise.
    pub seconds: i32,
    /// e.g. "Speeding In Pitlane".
    pub reason: String,
}

/// Lap timing derived by the recorder (not directly available from the game).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timing {
    pub current_lap_time: f64,
    pub last_lap_time: f64,
    pub last_lap_valid: bool,
    pub session_best: f64,
    /// All-time best for this track and car from the local database.
    pub personal_best: f64,
    pub personal_best_sectors: [f64; 3],
    pub session_best_sectors: [f64; 3],
    pub last_sectors: [f64; 3],
    pub current_sectors: [f64; 3],
    /// Live delta to the personal best lap at the same track position.
    pub delta_pb: Option<f64>,
    pub predicted_lap: Option<f64>,
    pub stint_laps: u32,
    pub stint_time: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Strategy {
    pub fuel_per_lap: f64,
    pub ve_per_lap: f64,
    pub laps_of_fuel: f64,
    pub laps_of_ve: f64,
    pub laps_remaining: f64,
    pub fuel_to_finish: f64,
    pub ve_to_finish: f64,
    pub avg_lap_time: f64,
    pub samples: u32,
}
