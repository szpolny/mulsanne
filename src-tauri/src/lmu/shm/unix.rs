//! Linux access to LMU's shared memory. LMU runs under Wine/Proton, and its
//! named file mappings live inside wineserver, so a native process can't open
//! them. A small bridge running inside the game's prefix mirrors the block into
//! a regular file instead (default `/dev/shm/LMU_Data`, override with
//! `MULSANNE_LMU_SHM`), rewriting it every frame and deleting it on exit.

use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
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
    path: PathBuf,
}

impl LmuSharedMemory {
    /// Opens the mirror if the bridge is running and the block is complete, `None` otherwise.
    pub fn open() -> Option<Self> {
        Self::open_path(&mirror_path())
    }

    fn open_path(path: &Path) -> Option<Self> {
        current_frame(path)?;
        Some(Self {
            path: path.to_path_buf(),
        })
    }

    /// The mirror has no events, so this just sleeps ~60 Hz.
    pub fn wait_for_frame(&self, _timeout_ms: u32) -> bool {
        std::thread::sleep(Duration::from_millis(16));
        true
    }

    /// The bridge publishes by atomic rename. Open each frame separately so
    /// every read sees one complete immutable snapshot, including after restart.
    pub fn copy_into(&self, dst: &mut [u8]) -> bool {
        assert_eq!(dst.len(), layout::TOTAL_SIZE);
        current_frame(&self.path).is_some_and(|file| file.read_exact_at(dst, 0).is_ok())
    }

    /// Deletion covers normal game exit; stale mtime covers a crashed bridge.
    pub fn game_alive(&self, _buf: &[u8]) -> bool {
        current_frame(&self.path).is_some()
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

/// Focus is sampled inside Wine and stored after the raw SDK block.
pub fn is_foreground(hwnd: u64) -> bool {
    hwnd != 0 && foreground_at(&mirror_path())
}

fn foreground_at(path: &Path) -> bool {
    let mut focused = [0];
    current_frame(path).is_some_and(|file| {
        file.read_exact_at(&mut focused, layout::TOTAL_SIZE as u64)
            .is_ok()
            && focused[0] == 1
    })
}

fn current_frame(path: &Path) -> Option<File> {
    let file = File::open(path).ok()?;
    let meta = file.metadata().ok()?;
    // One trailing byte carries focus; reject older/incomplete bridge output.
    if meta.len() != layout::TOTAL_SIZE as u64 + 1 {
        return None;
    }
    let age = SystemTime::now()
        .duration_since(meta.modified().ok()?)
        .ok()?;
    (age < STALE_AFTER).then_some(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{FileTimes, OpenOptions};
    use std::io::{Seek, SeekFrom, Write};

    struct Mirror(PathBuf);

    impl Mirror {
        fn new() -> Self {
            let id = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self(std::env::temp_dir().join(format!("mulsanne-shm-{}-{id}", std::process::id())))
        }

        fn write(&self, size: u64) -> File {
            let file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&self.0)
                .unwrap();
            file.set_len(size).unwrap();
            file
        }
    }

    impl Drop for Mirror {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn rejects_missing_incomplete_and_stale_mirrors() {
        let mirror = Mirror::new();
        assert!(LmuSharedMemory::open_path(&mirror.0).is_none());
        let file = mirror.write(10);
        assert!(LmuSharedMemory::open_path(&mirror.0).is_none());
        file.set_len(layout::TOTAL_SIZE as u64 + 1).unwrap();
        assert!(LmuSharedMemory::open_path(&mirror.0).is_some());
        file.set_times(FileTimes::new().set_modified(SystemTime::now() - STALE_AFTER * 2))
            .unwrap();
        assert!(LmuSharedMemory::open_path(&mirror.0).is_none());
    }

    #[test]
    fn detects_removal_and_reconnects_to_replacement() {
        let mirror = Mirror::new();
        mirror.write(layout::TOTAL_SIZE as u64 + 1);
        let mem = LmuSharedMemory::open_path(&mirror.0).unwrap();
        std::fs::remove_file(&mirror.0).unwrap();
        assert!(!mem.game_alive(&[]));
        mirror.write(layout::TOTAL_SIZE as u64 + 1);
        assert!(mem.game_alive(&[]));
        assert!(LmuSharedMemory::open_path(&mirror.0)
            .unwrap()
            .game_alive(&[]));
    }

    #[test]
    fn observes_updates_focus_and_expiry() {
        let mirror = Mirror::new();
        let mut file = mirror.write(layout::TOTAL_SIZE as u64 + 1);
        let mem = LmuSharedMemory::open_path(&mirror.0).unwrap();
        let mut buf = vec![0; layout::TOTAL_SIZE];
        for value in [1, 2] {
            file.seek(SeekFrom::Start(0)).unwrap();
            file.write_all(&[value]).unwrap();
            assert!(mem.copy_into(&mut buf));
            assert_eq!(buf[0], value);
            file.seek(SeekFrom::Start(layout::TOTAL_SIZE as u64))
                .unwrap();
            file.write_all(&[value % 2]).unwrap();
            assert_eq!(foreground_at(&mirror.0), value % 2 == 1);
        }
        file.set_times(FileTimes::new().set_modified(SystemTime::now() - STALE_AFTER * 2))
            .unwrap();
        assert!(!mem.game_alive(&buf));
    }

    #[test]
    fn concurrent_publication_never_mixes_frames() {
        let mirror = Mirror::new();
        std::fs::write(&mirror.0, vec![0; layout::TOTAL_SIZE + 1]).unwrap();
        let mem = LmuSharedMemory::open_path(&mirror.0).unwrap();
        let path = mirror.0.clone();
        let writer = std::thread::spawn(move || {
            let staging = path.with_extension("tmp");
            for value in 1..=100u8 {
                let mut frame = vec![value; layout::TOTAL_SIZE + 1];
                frame[layout::TOTAL_SIZE] = value % 2;
                std::fs::write(&staging, frame).unwrap();
                std::fs::rename(&staging, &path).unwrap();
            }
        });
        let mut buf = vec![0; layout::TOTAL_SIZE];
        for _ in 0..100 {
            assert!(mem.copy_into(&mut buf));
            assert!(buf.iter().all(|&byte| byte == buf[0]));
        }
        writer.join().unwrap();
        assert!(mem.copy_into(&mut buf));
        assert_eq!(buf[0], 100);
    }
}
