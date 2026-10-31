//! Runs inside the LMU Wine/Proton prefix and copies the game's shared memory
//! block into a regular file each frame. Native Linux Mulsanne reads that file
//! (`/dev/shm/LMU_Data` by default). Usage: `lmu-bridge.exe [mirror-path]`.
//! The path defaults to `$MULSANNE_LMU_SHM`, then `Z:\dev\shm\LMU_Data`.

#[cfg(not(windows))]
fn main() {
    eprintln!("lmu-bridge only runs under Windows or Wine; native Mulsanne reads its mirror file.");
    std::process::exit(1);
}

#[cfg(windows)]
mod lmu {
    // Share the exact layout and Win32 access code the Windows app uses.
    #[path = "../../../src-tauri/src/lmu/layout.rs"]
    pub mod layout;
    pub mod shm {
        #![allow(dead_code)]
        #[path = "../../../../src-tauri/src/lmu/shm/windows.rs"]
        mod windows;
        pub use windows::*;
    }
}

#[cfg(windows)]
fn main() {
    use std::fs::OpenOptions;
    use std::os::windows::fs::FileExt;
    use std::time::Duration;

    use lmu::{layout, shm};

    let path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("MULSANNE_LMU_SHM").ok())
        .unwrap_or_else(|| r"Z:\dev\shm\LMU_Data".to_string());
    let mut buf = vec![0u8; layout::TOTAL_SIZE];

    loop {
        let Some(mem) = shm::LmuSharedMemory::open() else {
            std::thread::sleep(Duration::from_secs(2));
            continue;
        };
        eprintln!("LMU found, mirroring to {path}");
        // Written in place so a reader that keeps the file open sees every update.
        let file = match OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("cannot open {path}: {e}");
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
        };
        let _ = file.set_len(layout::TOTAL_SIZE as u64);
        loop {
            mem.wait_for_frame(250);
            if mem.copy_into(&mut buf) {
                if !mem.game_alive(&buf) {
                    break;
                }
                if let Err(e) = file.seek_write(&buf, 0) {
                    eprintln!("write failed: {e}");
                    break;
                }
            }
        }
        drop(file);
        let _ = std::fs::remove_file(&path);
        eprintln!("LMU closed");
    }
}
