//! Publish immutable frames: rename on the same filesystem is atomic under Wine.
//! The raw SDK block is followed by one byte (0/1) for the Wine foreground state.
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct Mirror {
    path: PathBuf,
}

impl Mirror {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn publish(&self, frame: &[u8], focused: bool) -> io::Result<()> {
        let mut name = self.path.file_name().unwrap_or_default().to_os_string();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        name.push(format!(".{}.{nonce}.tmp", std::process::id()));
        let staging = self.path.with_file_name(name);
        // Close the staging handle before renaming it on Windows. Readers open
        // the published path anew each frame and can never observe a short write.
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staging)?;
            file.write_all(frame)?;
            file.write_all(&[u8::from(focused)])?;
            drop(file);
            // Wine/Windows can briefly deny replacement while another process
            // opens the destination. Keep the old complete frame during retries.
            for attempt in 0..20 {
                match fs::rename(&staging, &self.path) {
                    Err(e) if e.kind() == io::ErrorKind::PermissionDenied && attempt < 19 => {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    result => return result,
                }
            }
            unreachable!()
        })();
        if result.is_err() {
            let _ = fs::remove_file(&staging);
        }
        result
    }

    pub fn remove(&self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_complete_frames_and_preserves_open_readers() {
        let path = std::env::temp_dir().join(format!("mulsanne-publish-{}", std::process::id()));
        let mirror = Mirror::new(path);
        mirror.publish(&[1; 128], true).unwrap();
        let old = fs::read(mirror.path()).unwrap();
        assert_eq!(&old[..128], &[1; 128]);
        assert_eq!(old[128], 1);
        // Native readers may still have the previous inode open during rename.
        #[cfg(unix)]
        let mut reader = fs::File::open(mirror.path()).unwrap();
        mirror.publish(&[2; 128], false).unwrap();
        let new = fs::read(mirror.path()).unwrap();
        assert_eq!(&new[..128], &[2; 128]);
        assert_eq!(new[128], 0);
        #[cfg(unix)]
        {
            use std::io::Read;
            let mut previous = Vec::new();
            reader.read_to_end(&mut previous).unwrap();
            assert_eq!(previous, old);
        }
        mirror.remove();
        assert!(!mirror.path().exists());
    }
}
