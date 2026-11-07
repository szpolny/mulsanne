// Use the same wire layout and Win32 reader as the desktop app.
#![allow(dead_code)]

#[path = "../../src-tauri/src/lmu/layout.rs"]
pub mod layout;
#[path = "../../src-tauri/src/lmu/shm/windows.rs"]
pub mod shm;
