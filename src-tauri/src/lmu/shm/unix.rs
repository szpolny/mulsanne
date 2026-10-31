//! Linux access to LMU's shared memory. LMU runs under Wine/Proton, and its
//! named file mappings live inside wineserver, so a native process can't open
//! them. A small bridge running inside the game's prefix mirrors the block into
//! a regular file instead (default `/dev/shm/LMU_Data`, override with
//! `MULSANNE_LMU_SHM`), rewriting it every frame and deleting it on exit.

use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::lmu::layout::{self, ApplicationState};

/// The mirror counts as stale when the bridge hasn't touched it this long.
const STALE_AFTER: Duration = Duration::from_secs(10);

fn mirror_path() -> PathBuf {
    std::env::var_os("MULSANNE_LMU_SHM")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/dev/shm").join(layout::MAPPING_NAME))
}

/// An open connection to a running LMU instance.
pub struct LmuSharedMemory {
    file: File,
}

impl LmuSharedMemory {
    /// Opens the mirror if the bridge is running and the block is complete, `None` otherwise.
    pub fn open() -> Option<Self> {
        let file = File::open(mirror_path()).ok()?;
        (file.metadata().ok()?.len() >= layout::TOTAL_SIZE as u64).then_some(Self { file })
    }

    /// The mirror has no events, so this just sleeps ~60 Hz.
    pub fn wait_for_frame(&self, _timeout_ms: u32) -> bool {
        std::thread::sleep(Duration::from_millis(16));
        true
    }

    /// Copies the whole block into `dst`. The game's lock isn't reachable from
    /// here, so a read can rarely straddle a write; the next frame corrects it.
    pub fn copy_into(&self, dst: &mut [u8]) -> bool {
        assert_eq!(dst.len(), layout::TOTAL_SIZE);
        self.file.read_exact_at(dst, 0).is_ok()
    }

    /// The bridge deletes the mirror when the game exits; a stale mtime covers a crashed bridge.
    pub fn game_alive(&self, _buf: &[u8]) -> bool {
        let Ok(path_meta) = std::fs::metadata(mirror_path()) else { return false };
        path_meta
            .modified()
            .ok()
            .and_then(|t| SystemTime::now().duration_since(t).ok())
            .is_none_or(|age| age < STALE_AFTER)
    }
}

/// The game's main window handle as published in shared memory (0 if not created yet).
pub fn game_window(buf: &[u8]) -> u64 {
    let app: ApplicationState = layout::read_at(buf, layout::OFF_GENERIC_APP_INFO);
    app.app_window
}

/// Folder holding the game exe. A Wine window handle can't be resolved to a
/// process from here, so it comes from `MULSANNE_LMU_DIR` instead.
pub fn process_dir(hwnd: u64) -> Option<PathBuf> {
    if hwnd == 0 {
        return None;
    }
    std::env::var_os("MULSANNE_LMU_DIR").map(PathBuf::from)
}

/// Window focus under Wine isn't observable from a native process, so treat the game as always focused.
pub fn is_foreground(_hwnd: u64) -> bool {
    true
}
