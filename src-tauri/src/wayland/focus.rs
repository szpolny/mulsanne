use std::collections::HashMap;

use wayland_client::backend::ObjectId;
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1 as handle, zwlr_foreign_toplevel_manager_v1 as manager,
};

/// The Wine bridge sees focus inside Gamescope. The desktop compositor sees
/// whether the outer Gamescope window still has focus.
pub struct Tracker {
    queue: EventQueue<Windows>,
    windows: Windows,
}

impl Tracker {
    pub fn new() -> Option<Self> {
        if !super::active() {
            return None;
        }
        let connection = Connection::connect_to_env().ok()?;
        let (globals, queue) = registry_queue_init::<Windows>(&connection).ok()?;
        globals
            .bind::<manager::ZwlrForeignToplevelManagerV1, _, _>(&queue.handle(), 1..=1, ())
            .ok()?;
        Some(Self {
            queue,
            windows: Windows::default(),
        })
    }

    /// None keeps the bridge as the focus source for direct launches and on
    /// compositors where the window list becomes unavailable.
    pub fn game_focused(&mut self) -> Option<bool> {
        if self.windows.finished || self.queue.roundtrip(&mut self.windows).is_err() {
            self.windows.finished = true;
            return None;
        }
        self.windows.game_focused()
    }
}

#[derive(Default)]
struct Windows {
    toplevels: HashMap<ObjectId, Toplevel>,
    finished: bool,
}

#[derive(Default)]
struct Toplevel {
    app_id: String,
    title: String,
    active: bool,
}

impl Toplevel {
    fn is_game(&self) -> bool {
        self.app_id.eq_ignore_ascii_case("gamescope")
            && (self.title == "Le Mans Ultimate" || self.title.starts_with("Le Mans Ultimate v"))
    }
}

impl Windows {
    fn game_focused(&self) -> Option<bool> {
        let mut windows = self.toplevels.values().filter(|window| window.is_game());
        let game = windows.next()?;
        Some(game.active || windows.any(|window| window.active))
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Windows {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<manager::ZwlrForeignToplevelManagerV1, ()> for Windows {
    wayland_client::event_created_child!(Windows, manager::ZwlrForeignToplevelManagerV1, [
        manager::EVT_TOPLEVEL_OPCODE => (handle::ZwlrForeignToplevelHandleV1, ())
    ]);

    fn event(
        windows: &mut Self,
        _: &manager::ZwlrForeignToplevelManagerV1,
        event: manager::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let manager::Event::Finished = event {
            windows.finished = true;
            windows.toplevels.clear();
        }
    }
}

impl Dispatch<handle::ZwlrForeignToplevelHandleV1, ()> for Windows {
    fn event(
        windows: &mut Self,
        proxy: &handle::ZwlrForeignToplevelHandleV1,
        event: handle::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let handle::Event::Closed = event {
            windows.toplevels.remove(&proxy.id());
            proxy.destroy();
            return;
        }
        let window = windows.toplevels.entry(proxy.id()).or_default();
        match event {
            handle::Event::AppId { app_id } => window.app_id = app_id,
            handle::Event::Title { title } => window.title = title,
            handle::Event::State { state } => {
                window.active = state
                    .chunks_exact(4)
                    .any(|value| value == (handle::State::Activated as u32).to_ne_bytes());
            }
            _ => {}
        }
    }
}
