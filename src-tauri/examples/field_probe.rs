//! Dumps per-car telemetry (tyres, track limits) for every active vehicle.
//! `cargo run --example field_probe`

use lm_overlay_lib::lmu::layout::{self, cstr, read_at, ScoringInfo, TelemInfo, VehicleScoring};
use lm_overlay_lib::lmu::shm::LmuSharedMemory;

fn main() {
    let mem = LmuSharedMemory::open().expect("LMU not running");
    let mut buf = vec![0u8; layout::TOTAL_SIZE];
    mem.wait_for_frame(500);
    mem.copy_into(&mut buf);
    let info: ScoringInfo = read_at(&buf, layout::OFF_SCORING_INFO);
    let (tl_pen, tl_pt) = (info.track_limits_steps_per_penalty, info.track_limits_steps_per_point);
    println!("track={} session={} steps/penalty={} steps/point={}", cstr(&info.track_name), info.session, tl_pen, tl_pt);
    let active: u8 = read_at(&buf, layout::OFF_TELEMETRY_ACTIVE_VEHICLES);
    let player: u8 = read_at(&buf, layout::OFF_TELEMETRY_PLAYER_IDX);
    println!("active telemetry vehicles={active} playerIdx={player} scoring vehicles={}", info.num_vehicles);
    for i in 0..(active as usize).min(layout::MAX_VEHICLES) {
        let t: TelemInfo = read_at(&buf, layout::OFF_TELEMETRY_INFO + i * std::mem::size_of::<TelemInfo>());
        let w = t.wheels;
        let wear: Vec<String> = w.iter().map(|x| format!("{:.3}", { x.wear })).collect();
        let comp: Vec<String> = w.iter().map(|x| format!("{}/{}", x.compound_index, x.compound_type)).collect();
        let id = t.id;
        println!(
            "#{i:02} id={id} {:?} model={:?} class={} champ={} F={:?} R={:?} idx={}/{} wear={:?} comp={:?} tlSteps={} lapInv={}",
            cstr(&t.vehicle_name), cstr(&t.vehicle_model), t.vehicle_class, t.vehicle_championship,
            cstr(&t.front_tire_compound_name), cstr(&t.rear_tire_compound_name),
            t.front_tire_compound_index, t.rear_tire_compound_index, wear, comp, t.track_limits_steps, t.lap_invalidated
        );
    }
    for i in 0..(info.num_vehicles.max(0) as usize).min(5) {
        let v: VehicleScoring = read_at(&buf, layout::OFF_SCORING_VEHICLES + i * std::mem::size_of::<VehicleScoring>());
        println!("scoring id={} {:?} veh={:?} file={:?} pits={} penalties={}", { v.id }, cstr(&v.driver_name), cstr(&v.vehicle_name), cstr(&v.veh_filename), { v.num_pitstops }, { v.num_penalties });
    }
}
