//! Fetches the LMU daily race schedule from RaceControl and prints it.
//! Run with `cargo run --example schedule_probe`.

fn main() {
    match tauri::async_runtime::block_on(lm_overlay_lib::schedule::fetch_daily_races()) {
        Ok(races) => {
            for race in races {
                println!(
                    "[{}] {} | {} | {} | {} starts {:?}",
                    race.tier, race.name, race.track, race.duration, race.image.is_some(), race.starts
                );
            }
        }
        Err(e) => eprintln!("error: {e}"),
    }
}
