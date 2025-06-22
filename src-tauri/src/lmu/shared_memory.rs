use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{
    FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile, OpenFileMappingA,
};
use windows::core::PCSTR;

pub struct SharedMemory {
    pub handle: HANDLE,
    pub view_ptr: MEMORY_MAPPED_VIEW_ADDRESS,
}

impl SharedMemory {
    pub fn new(memory_name: &str) -> Self {
        let handle;

        loop {
            let mapping_name = PCSTR(format!("{memory_name}\0").as_bytes().as_ptr());

            let handler_opt =
                unsafe { OpenFileMappingA(FILE_MAP_READ.0, false, mapping_name) }.ok();

            if let Some(val) = handler_opt {
                handle = val;
                break;
            }
        }

        let view_ptr = unsafe { MapViewOfFile(handle, FILE_MAP_READ, 0, 0, 0) };

        Self { handle, view_ptr }
    }

    pub fn get_as<T>(&self) -> &T {
        unsafe { &*(self.view_ptr.Value as *const T) }
    }

    pub fn get_view_ptr(&self) -> MEMORY_MAPPED_VIEW_ADDRESS {
        self.view_ptr
    }

    pub fn get_handler(&self) -> HANDLE {
        self.handle
    }
}
