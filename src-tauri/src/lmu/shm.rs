//! Win32 access to the native LMU shared memory block, its update events and
//! the cooperative spin lock the game uses while writing.

use std::ffi::CString;
use std::sync::atomic::{AtomicI32, Ordering};

use windows::core::PCSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, INVALID_HANDLE_VALUE, WAIT_OBJECT_0};
use windows::Win32::System::Memory::{
    CreateFileMappingA, MapViewOfFile, OpenFileMappingA, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
    FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
};
use windows::Win32::System::Threading::{
    CreateEventA, OpenEventA, SetEvent, WaitForSingleObject, SYNCHRONIZATION_SYNCHRONIZE,
};
use windows::Win32::UI::WindowsAndMessaging::IsWindow;

use super::layout::{self, ApplicationState};

fn c_name(name: &str) -> CString {
    CString::new(name).expect("static name without NUL")
}

struct MappedView {
    handle: HANDLE,
    view: MEMORY_MAPPED_VIEW_ADDRESS,
}

impl MappedView {
    fn as_ptr(&self) -> *const u8 {
        self.view.Value as *const u8
    }
}

impl Drop for MappedView {
    fn drop(&mut self) {
        unsafe {
            let _ = UnmapViewOfFile(self.view);
            let _ = CloseHandle(self.handle);
        }
    }
}

struct OwnedEvent(HANDLE);

impl OwnedEvent {
    fn open(name: &str) -> Option<Self> {
        let n = c_name(name);
        unsafe { OpenEventA(SYNCHRONIZATION_SYNCHRONIZE, false, PCSTR(n.as_ptr() as _)) }
            .ok()
            .map(OwnedEvent)
    }

    fn wait(&self, timeout_ms: u32) -> bool {
        unsafe { WaitForSingleObject(self.0, timeout_ms) == WAIT_OBJECT_0 }
    }
}

impl Drop for OwnedEvent {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Client side of `SharedMemoryLock` from the SDK. The game and every reader
/// share an 8-byte `{ waiters, busy }` block plus an auto-reset event.
struct SharedLock {
    view: MappedView,
    event: HANDLE,
}

impl SharedLock {
    fn open() -> Option<Self> {
        let map_name = c_name(layout::LOCK_MAPPING_NAME);
        let event_name = c_name(layout::LOCK_EVENT_NAME);
        unsafe {
            let handle = CreateFileMappingA(
                INVALID_HANDLE_VALUE,
                None,
                PAGE_READWRITE,
                0,
                8,
                PCSTR(map_name.as_ptr() as _),
            )
            .ok()?;
            let view = MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, 8);
            if view.Value.is_null() {
                let _ = CloseHandle(handle);
                return None;
            }
            let view = MappedView { handle, view };
            let event = CreateEventA(None, false, false, PCSTR(event_name.as_ptr() as _)).ok()?;
            Some(Self { view, event })
        }
    }

    fn waiters(&self) -> &AtomicI32 {
        unsafe { &*(self.view.as_ptr() as *const AtomicI32) }
    }

    fn busy(&self) -> &AtomicI32 {
        unsafe { &*(self.view.as_ptr().add(4) as *const AtomicI32) }
    }

    fn try_acquire(&self) -> bool {
        self.busy().compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }

    /// Acquires the lock, giving up after roughly `timeout_ms` so a stuck
    /// writer can never freeze our reader thread.
    fn lock(&self, timeout_ms: u32) -> Option<SharedLockGuard<'_>> {
        for _ in 0..4000 {
            if self.try_acquire() {
                return Some(SharedLockGuard(self));
            }
            std::hint::spin_loop();
        }
        self.waiters().fetch_add(1, Ordering::AcqRel);
        let mut acquired = false;
        let mut waited = 0;
        while waited <= timeout_ms {
            if self.try_acquire() {
                acquired = true;
                break;
            }
            unsafe { WaitForSingleObject(self.event, 5) };
            waited += 5;
        }
        self.waiters().fetch_sub(1, Ordering::AcqRel);
        acquired.then_some(SharedLockGuard(self))
    }

    fn unlock(&self) {
        self.busy().store(0, Ordering::Release);
        if self.waiters().load(Ordering::Acquire) > 0 {
            unsafe {
                let _ = SetEvent(self.event);
            }
        }
    }
}

impl Drop for SharedLock {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.event);
        }
    }
}

struct SharedLockGuard<'a>(&'a SharedLock);

impl Drop for SharedLockGuard<'_> {
    fn drop(&mut self) {
        self.0.unlock();
    }
}

/// An open connection to a running LMU instance.
pub struct LmuSharedMemory {
    data: MappedView,
    lock: Option<SharedLock>,
    hold_event: Option<OwnedEvent>,
    data_event: Option<OwnedEvent>,
}

// The raw handles are only used from the thread that owns the reader.
unsafe impl Send for LmuSharedMemory {}

impl LmuSharedMemory {
    /// Opens the shared memory if the game is running, `None` otherwise.
    pub fn open() -> Option<Self> {
        let name = c_name(layout::MAPPING_NAME);
        unsafe {
            let handle = OpenFileMappingA(FILE_MAP_READ.0, false, PCSTR(name.as_ptr() as _)).ok()?;
            let view = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, layout::TOTAL_SIZE);
            if view.Value.is_null() {
                let _ = CloseHandle(handle);
                return None;
            }
            Some(Self {
                data: MappedView { handle, view },
                lock: SharedLock::open(),
                hold_event: OwnedEvent::open(layout::HOLD_EVENT_NAME),
                data_event: OwnedEvent::open(layout::DATA_EVENT_NAME),
            })
        }
    }

    /// Blocks until the game publishes a new frame. Returns `false` on timeout.
    /// Older game builds have no events; then this just sleeps ~60 Hz.
    pub fn wait_for_frame(&self, timeout_ms: u32) -> bool {
        match (&self.hold_event, &self.data_event) {
            (Some(hold), Some(data)) => hold.wait(timeout_ms) && data.wait(timeout_ms),
            _ => {
                std::thread::sleep(std::time::Duration::from_millis(16));
                true
            }
        }
    }

    /// Copies the whole block into `dst` while holding the game's lock.
    pub fn copy_into(&self, dst: &mut [u8]) -> bool {
        assert_eq!(dst.len(), layout::TOTAL_SIZE);
        let _guard = match &self.lock {
            Some(lock) => match lock.lock(100) {
                Some(guard) => Some(guard),
                None => return false,
            },
            None => None,
        };
        unsafe {
            std::ptr::copy_nonoverlapping(self.data.as_ptr(), dst.as_mut_ptr(), layout::TOTAL_SIZE);
        }
        true
    }

    /// The game keeps the mapping alive as long as we hold a handle, so use the
    /// game window to detect that it has exited.
    pub fn game_alive(&self, buf: &[u8]) -> bool {
        let app: ApplicationState = layout::read_at(buf, layout::OFF_GENERIC_APP_INFO);
        let hwnd = app.app_window;
        if hwnd == 0 {
            // Window not created yet (still starting up); assume alive.
            return true;
        }
        unsafe { IsWindow(Some(HWND(hwnd as *mut _))).as_bool() }
    }
}
