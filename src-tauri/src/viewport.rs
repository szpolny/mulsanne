//! Follow LMU's client rectangle, including moves between monitors.
mod hyprland;

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
    x11: Option<X11>,
    compositor: Option<hyprland::Compositor>,
    previous: Option<(Bounds, bool)>,
}

struct X11 {
    connection: RustConnection,
    root: u32,
    clients: u32,
}

impl X11 {
    fn new() -> Option<Self> {
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
    /// The game's rectangle in the compositor's logical layout. XWayland's
    /// RandR monitors carry the compositor's output names but not always its
    /// positions or scale, so the rectangle is mapped through the monitor it's on.
    fn logical(&self, b: Bounds) -> [i32; 4] {
        let size = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
        let raw = [b.x, b.y, size(b.width), size(b.height)];
        let (cx, cy) = (b.x + raw[2] / 2, b.y + raw[3] / 2);
        let Some((name, monitor)) = self.monitors().into_iter().find(|(_, [x, y, w, h])| {
            (*x..x + w).contains(&cx) && (*y..y + h).contains(&cy)
        }) else {
            return raw;
        };
        let Some(output) = crate::wayland::output_layout().into_iter().find(|o| o.name == name) else {
            return raw;
        };
        let scale = |v: i32, from: i32, to: i32| (i64::from(v) * i64::from(to) / i64::from(from.max(1))) as i32;
        let [ox, oy, ow, oh] = output.rect;
        let [mx, my, mw, mh] = monitor;
        [
            ox + scale(b.x - mx, mw, ow),
            oy + scale(b.y - my, mh, oh),
            scale(raw[2], mw, ow),
            scale(raw[3], mh, oh),
        ]
    }

    /// XWayland's RandR monitors: connector name and rectangle.
    fn monitors(&self) -> Vec<(String, [i32; 4])> {
        use x11rb::protocol::randr::ConnectionExt as _;
        let Some(reply) = self.connection.randr_get_monitors(self.root, true).ok().and_then(|c| c.reply().ok()) else {
            return Vec::new();
        };
        reply
            .monitors
            .iter()
            .filter_map(|m| {
                let name = self.connection.get_atom_name(m.name).ok()?.reply().ok()?.name;
                let rect = [m.x.into(), m.y.into(), m.width.into(), m.height.into()];
                Some((String::from_utf8_lossy(&name).into_owned(), rect))
            })
            .collect()
    }
}

impl Tracker {
    pub fn new() -> Self {
        let compositor = if crate::wayland::active() {
            hyprland::Compositor::new()
        } else {
            None
        };
        Self {
            x11: X11::new(),
            compositor,
            previous: None,
        }
    }

    /// Hyprland can see the outer Gamescope window, which is absent from the
    /// desktop's X11 client list. Focus inside Gamescope is independent of it.
    pub fn update(&mut self, app: &AppHandle) -> Option<bool> {
        if let Some((bounds, focused)) = self
            .compositor
            .as_ref()
            .and_then(hyprland::Compositor::game_window)
        {
            self.place(app, bounds, true);
            return Some(focused);
        }
        let bounds = self.x11.as_ref().and_then(X11::bounds);
        self.place(app, bounds, false);
        None
    }

    fn place(&mut self, app: &AppHandle, bounds: Option<Bounds>, logical: bool) {
        let Some(bounds) = bounds else {
            self.previous = None;
            return;
        };
        let placement = Some((bounds, logical));
        if self.previous == placement {
            return;
        }
        if crate::wayland::active() {
            let [x, y, width, height] = if logical {
                [bounds.x, bounds.y, bounds.width as i32, bounds.height as i32]
            } else if let Some(x11) = &self.x11 {
                x11.logical(bounds)
            } else {
                return;
            };
            let app_ = app.clone();
            if app
                .run_on_main_thread(move || {
                    if let Some(window) = app_.get_webview_window("overlay") {
                        if let Ok(gtk_window) = window.gtk_window() {
                            crate::wayland::follow(&gtk_window, x, y, width, height);
                        }
                        let editing = app_.state::<crate::AppState>().edit_mode
                            .load(std::sync::atomic::Ordering::Acquire);
                        let _ = window.set_ignore_cursor_events(!editing);
                    }
                })
                .is_ok()
            {
                self.previous = placement;
            }
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
                // GTK can reset input regions when the client is resized.
                let editing = app.state::<crate::AppState>().edit_mode
                    .load(std::sync::atomic::Ordering::Acquire);
                let _ = window.set_ignore_cursor_events(!editing);
                self.previous = placement;
            }
        }
    }
}
