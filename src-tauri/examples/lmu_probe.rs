//! Prints what the app reads from LMU's shared memory, once per second.
//! Run with the game on track: `cargo run --example lmu_probe`

use std::time::Duration;

use lm_overlay_lib::lmu::{self, layout, shm::LmuSharedMemory};

fn main() {
    let mem = loop {
        match LmuSharedMemory::open() {
            Some(m) => break m,
            None => {
                println!("LMU_Data not found, is the game running?");
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    };
    let mut buf = vec![0u8; layout::TOTAL_SIZE];
    for _ in 0..6 {
        let frame = mem.wait_for_frame(500);
        let copied = mem.copy_into(&mut buf);
        let s = lmu::parse(&buf);
        println!(
            "frame={frame} copied={copied} alive={} ver={} realtime={} track={:?} len={:.0} session={:?} phase={} et={:.1} remaining={:.0} cars={} yellow={} sectorFlags={:?} playerFlag={:?}",
            mem.game_alive(&buf),
            s.game_version,
            s.in_realtime,
            s.session.track,
            s.session.track_length,
            s.session.kind,
            s.session.phase,
            s.session.current_et,
            s.session.time_remaining,
            s.vehicles.len(),
            s.session.yellow_flag_state,
            s.session.sector_flags_raw,
            s.vehicles.iter().find(|v| v.is_player).map(|v| v.flag)
        );
        match &s.player {
            Some(p) => println!(
                "  player {:?}/{:?} class={:?} lap={} lapStart={:.2} et={:.2} dist={:.0} sector={} pits={} speed={:.1}km/h gear={} rpm={:.0} thr={:.2} brk={:.2} fuel={:.2} ve={:.3} dBest={:.3} tc={}/{} abs={}/{} tyreFL={:.0}C {:.0}kPa wear={:.2}",
                p.vehicle_model, p.vehicle_name, p.vehicle_class, p.lap_number, p.lap_start_et, p.elapsed_time, p.lap_dist,
                p.sector, p.in_pits, p.speed * 3.6, p.gear, p.rpm, p.throttle, p.brake, p.fuel, p.virtual_energy, p.delta_best,
                p.tc, p.tc_max, p.abs, p.abs_max, p.wheels[0].surface_temp[1], p.wheels[0].pressure, p.wheels[0].wear
            ),
            None => println!("  no player vehicle"),
        }
        if let Some(v) = s.vehicles.iter().find(|v| v.is_player) {
            println!(
                "  scoring P{} classP{} laps={} best={:.3} last={:.3} s1={:.3} s2={:.3} est={:.2}",
                v.place, v.class_place, v.total_laps, v.best_lap, v.last_lap, v.last_s1, v.last_s2, v.estimated_lap_time
            );
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}
