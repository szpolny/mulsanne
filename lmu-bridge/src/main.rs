//! Runs inside the LMU Wine/Proton prefix and copies the game's shared memory
//! block into a regular file each frame. Native Linux Mulsanne reads that file
//! (`/dev/shm/LMU_Data` by default). Usage: `lmu-bridge.exe [mirror-path]`.
//! The path defaults to `$MULSANNE_LMU_SHM`, then `Z:\dev\shm\LMU_Data`.

#[cfg(not(windows))]
fn main() {
    eprintln!("lmu-bridge only runs under Windows or Wine; native Mulsanne reads its mirror file.");
    std::process::exit(1);
}

#[cfg(any(windows, test))]
mod mirror;

#[cfg(windows)]
mod lmu;

#[cfg(windows)]
fn main() -> std::io::Result<()> {
    use std::time::Duration;

    use lmu::{layout, shm};

    let path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("MULSANNE_LMU_SHM").ok())
        .unwrap_or_else(|| r"Z:\dev\shm\LMU_Data".to_string());
    let mirror = mirror::Mirror::new(path.into());
    let mut buf = vec![0u8; layout::TOTAL_SIZE];

    let mem = loop {
        if let Some(mem) = shm::LmuSharedMemory::open() {
            break mem;
        }
        std::thread::sleep(Duration::from_secs(2));
    };
    eprintln!("LMU found, mirroring to {}", mirror.path().display());
    loop {
        mem.wait_for_frame(250);
        // Check the last known HWND even when the cooperative game lock fails.
        if !mem.game_alive(&buf) {
            break;
        }
        if mem.copy_into(&mut buf) {
            if !mem.game_alive(&buf) {
                break;
            }
            let focused = shm::is_foreground(shm::game_window(&buf));
            if let Err(e) = mirror.publish(&buf, focused) {
                mirror.remove();
                return Err(e);
            }
        }
    }
    mirror.remove();
    // Leaving a Wine process alive can prevent Steam's waitforexitandrun from
    // starting the next game session. One bridge invocation serves one session.
    eprintln!("LMU closed");
    Ok(())
}
