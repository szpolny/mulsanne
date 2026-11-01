//! Follow LMU's X11 client rectangle, including moves between monitors.
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ConnectionExt, MapState},
    rust_connection::RustConnection,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Bounds {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

pub struct Tracker {
    connection: RustConnection,
    root: u32,
    clients: u32,
    previous: Option<Bounds>,
}
impl Tracker {
    pub fn new() -> Option<Self> {
        let (connection, screen) = x11rb::connect(None).ok()?;
        let root = connection.setup().roots[screen].root;
        let clients = connection
            .intern_atom(false, b"_NET_CLIENT_LIST")
            .ok()?
            .reply()
            .ok()?
            .atom;
        Some(Self {
            connection,
            root,
            clients,
            previous: None,
        })
    }
    fn bounds(&self) -> Option<Bounds> {
        let reply = self
            .connection
            .get_property(
                false,
                self.root,
                self.clients,
                AtomEnum::WINDOW,
                0,
                u32::MAX,
            )
            .ok()?
            .reply()
            .ok()?;
        let bounds = reply
            .value32()?
            .filter_map(|window| {
                let class = self
                    .connection
                    .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 256)
                    .ok()?
                    .reply()
                    .ok()?;
                if !class
                    .value
                    .split(|b| *b == 0)
                    .any(|s| s == b"steam_app_2399420")
                {
                    return None;
                }
                if self
                    .connection
                    .get_window_attributes(window)
                    .ok()?
                    .reply()
                    .ok()?
                    .map_state
                    != MapState::VIEWABLE
                {
                    return None;
                }
                let geometry = self.connection.get_geometry(window).ok()?.reply().ok()?;
                let pos = self
                    .connection
                    .translate_coordinates(window, self.root, 0, 0)
                    .ok()?
                    .reply()
                    .ok()?;
                Some(Bounds {
                    x: pos.dst_x.into(),
                    y: pos.dst_y.into(),
                    width: geometry.width.into(),
                    height: geometry.height.into(),
                })
            })
            .max_by_key(|b| u64::from(b.width) * u64::from(b.height));
        bounds
    }
    pub fn update(&mut self, app: &AppHandle) {
        let Some(bounds) = self.bounds() else {
            return;
        };
        if self.previous == Some(bounds) {
            return;
        }
        if let Some(window) = app.get_webview_window("overlay") {
            if window
                .set_position(PhysicalPosition::new(bounds.x, bounds.y))
                .is_ok()
                && window
                    .set_size(PhysicalSize::new(bounds.width, bounds.height))
                    .is_ok()
            {
                self.previous = Some(bounds);
            }
        }
    }
}
