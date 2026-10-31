//! Runs natively on Windows, or under Wine in a disposable prefix.
#![cfg(windows)]

#[path = "../src/lmu.rs"]
mod lmu;

use std::fs;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use lmu::layout;
use windows::core::{s, w};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{
    CreateFileMappingA, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
    MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, WINDOW_EX_STYLE, WINDOW_STYLE,
};

struct Game {
    handle: HANDLE,
    view: MEMORY_MAPPED_VIEW_ADDRESS,
    window: HWND,
}

impl Game {
    fn new(version: i32) -> Self {
        unsafe {
            let window = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("LMU bridge test"),
                WINDOW_STYLE::default(),
                0,
                0,
                100,
                100,
                None,
                None,
                None,
                None,
            )
            .unwrap();
            let handle = CreateFileMappingA(
                INVALID_HANDLE_VALUE,
                None,
                PAGE_READWRITE,
                0,
                layout::TOTAL_SIZE as u32,
                s!("LMU_Data"),
            )
            .unwrap();
            let view = MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, layout::TOTAL_SIZE);
            assert!(!view.Value.is_null());
            std::ptr::write_bytes(view.Value as *mut u8, 0, layout::TOTAL_SIZE);
            std::ptr::write_unaligned(
                (view.Value as *mut u8).add(layout::OFF_GENERIC_APP_INFO) as *mut u64,
                window.0 as u64,
            );
            let game = Self {
                handle,
                view,
                window,
            };
            game.set_version(version);
            game
        }
    }

    fn set_version(&self, version: i32) {
        unsafe {
            std::ptr::write_volatile(
                (self.view.Value as *mut u8).add(layout::OFF_GENERIC_GAME_VERSION) as *mut i32,
                version,
            );
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.window).unwrap();
            UnmapViewOfFile(self.view).unwrap();
            CloseHandle(self.handle).unwrap();
        }
    }
}

struct Bridge(Child);

impl Drop for Bridge {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn wait_for(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !condition() {
        assert!(Instant::now() < deadline, "bridge lifecycle timed out");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn waits_streams_and_exits_with_the_game() {
    let path = std::env::var_os("MULSANNE_BRIDGE_TEST_MIRROR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("mulsanne-lifecycle-{}", std::process::id()))
        });
    let _ = fs::remove_file(&path);
    let mut bridge = Bridge(
        Command::new(env!("CARGO_BIN_EXE_lmu-bridge"))
            .arg(&path)
            .spawn()
            .unwrap(),
    );
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        !path.exists(),
        "no mirror should be published before the game starts"
    );
    let matches = |version| {
        fs::read(&path).is_ok_and(|buf| {
            buf.len() == layout::TOTAL_SIZE + 1
                && layout::read_at::<i32>(&buf, layout::OFF_GENERIC_GAME_VERSION) == version
                && buf[layout::TOTAL_SIZE] <= 1
        })
    };
    let game = Game::new(111);
    wait_for(|| matches(111));
    std::thread::sleep(Duration::from_millis(100));
    game.set_version(222);
    wait_for(|| matches(222));
    std::thread::sleep(Duration::from_millis(100));
    drop(game);
    wait_for(|| !path.exists());
    wait_for(|| bridge.0.try_wait().unwrap().is_some());
    assert!(bridge.0.wait().unwrap().success());
    // A fresh invocation can attach after a game restart.
    let game = Game::new(333);
    let mut bridge = Bridge(
        Command::new(env!("CARGO_BIN_EXE_lmu-bridge"))
            .arg(&path)
            .spawn()
            .unwrap(),
    );
    wait_for(|| matches(333));
    std::thread::sleep(Duration::from_millis(100));
    drop(game);
    wait_for(|| !path.exists());
    wait_for(|| bridge.0.try_wait().unwrap().is_some());
    assert!(bridge.0.wait().unwrap().success());
}
