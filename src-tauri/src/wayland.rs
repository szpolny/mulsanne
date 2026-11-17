//! Native Wayland HUD on compositors with wlr-layer-shell.
//!
//! On a Wayland session the companion and the HUD render natively, and the HUD
//! is a layer-shell surface on the overlay layer: drawn above fullscreen
//! windows, never tiled or decorated, on a chosen monitor. Compositors without
//! layer shell (GNOME) and X11 sessions keep the X11 window path.
use std::sync::atomic::{AtomicBool, Ordering};

use gtk::prelude::*;
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

static ACTIVE: AtomicBool = AtomicBool::new(false);

/// Whether the HUD is a layer-shell surface.
pub fn active() -> bool {
    ACTIVE.load(Ordering::Acquire)
}

/// Chooses the GDK backend before Tauri initializes GTK. Wayland sessions use
/// the Wayland backend when the compositor supports layer shell; otherwise the
/// app restarts itself under XWayland.
pub fn select_backend() {
    let chosen = std::env::var_os("GDK_BACKEND");
    if chosen.is_none() && std::env::var_os("WAYLAND_DISPLAY").is_some() {
        std::env::set_var("GDK_BACKEND", "wayland");
    } else if chosen.as_deref() != Some("wayland".as_ref()) {
        return;
    }
    if gtk::init().is_ok() && gtk_layer_shell::is_supported() {
        ACTIVE.store(true, Ordering::Release);
        return;
    }
    if chosen.is_some() {
        eprintln!("GDK_BACKEND=wayland, but the compositor has no layer shell; the HUD can't stay above the game");
        return;
    }
    // GTK can't switch backends once initialized, so start over on X11.
    use std::os::unix::process::CommandExt;
    let error = std::env::current_exe().map(|exe| {
        std::process::Command::new(exe)
            .args(std::env::args_os().skip(1))
            .env("GDK_BACKEND", "x11")
            .exec()
    });
    eprintln!("restarting under XWayland: {error:?}");
    std::process::exit(1);
}

/// Turns the unrealized HUD window into a full-monitor surface on the overlay layer.
pub fn init_overlay(window: &gtk::ApplicationWindow) {
    if window.is_realized() {
        window.unrealize();
    }
    // Tao gives undecorated Wayland windows an empty titlebar for client-side
    // decorations; a layer surface has none.
    window.set_titlebar(None::<&gtk::Widget>);
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace("mulsanne-hud");
    for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
        window.set_anchor(edge, true);
    }
    // Cover the whole monitor, ignoring panels' exclusive zones.
    window.set_exclusive_zone(-1);
    window.set_keyboard_mode(KeyboardMode::None);
}

/// Covers `monitor` (matched by GDK model, as Tauri names monitors; the
/// primary monitor when unmatched).
pub fn place(window: &gtk::ApplicationWindow, monitor: &str) {
    let display = WidgetExt::display(window);
    let found = (0..display.n_monitors())
        .filter_map(|i| display.monitor(i))
        .find(|m| m.model().is_some_and(|model| model == monitor))
        .or_else(|| display.primary_monitor())
        .or_else(|| display.monitor(0));
    if let Some(m) = found {
        window.set_monitor(&m);
    }
    set_margins(window, [0; 4]);
}

/// Covers the game's rectangle, given in the compositor's logical layout (see
/// [`output_layout`]).
pub fn follow(window: &gtk::ApplicationWindow, x: i32, y: i32, width: i32, height: i32) {
    let display = WidgetExt::display(window);
    let Some(monitor) = display.monitor_at_point(x + width / 2, y + height / 2) else { return };
    let area = monitor.geometry();
    window.set_monitor(&monitor);
    // Rounding can leave a fullscreen game a pixel past the monitor's edges;
    // negative insets clamp to zero.
    set_margins(window, [
        x - area.x(),
        area.x() + area.width() - (x + width),
        y - area.y(),
        area.y() + area.height() - (y + height),
    ]);
}

/// Left, right, top and bottom insets from the monitor's edges.
fn set_margins(window: &gtk::ApplicationWindow, margins: [i32; 4]) {
    for (edge, margin) in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom].into_iter().zip(margins) {
        window.set_layer_shell_margin(edge, margin.max(0));
    }
}

/// Lets the HUD take keyboard focus while the layout is being edited.
pub fn set_keyboard(window: &gtk::ApplicationWindow, editing: bool) {
    let mode = if editing { KeyboardMode::OnDemand } else { KeyboardMode::None };
    window.set_keyboard_mode(mode);
}

/// An output's connector name (e.g. "DP-1") and its rectangle in the
/// compositor's logical layout.
pub struct Output {
    pub name: String,
    pub rect: [i32; 4],
}

/// Outputs as the compositor lays them out, from xdg-output. XWayland names its
/// RandR monitors after the same connectors but may place them elsewhere (Hyprland
/// packs them from the origin), so game coordinates are mapped through these.
pub fn output_layout() -> Vec<Output> {
    use wayland_client::globals::registry_queue_init;
    use wayland_client::protocol::wl_output::WlOutput;
    use wayland_client::Connection;
    use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1::ZxdgOutputManagerV1;

    let Ok(conn) = Connection::connect_to_env() else { return Vec::new() };
    let Ok((globals, mut queue)) = registry_queue_init::<Layout>(&conn) else { return Vec::new() };
    let qh = queue.handle();
    // xdg-output sends output names from version 2.
    let Ok(manager) = globals.bind::<ZxdgOutputManagerV1, _, _>(&qh, 2..=3, ()) else { return Vec::new() };
    let mut layout = Layout::default();
    for global in globals.contents().clone_list() {
        if global.interface == "wl_output" {
            let output: WlOutput = globals.registry().bind(global.name, 1, &qh, ());
            manager.get_xdg_output(&output, &qh, layout.0.len());
            layout.0.push((None, [0; 4]));
        }
    }
    if queue.roundtrip(&mut layout).is_err() {
        return Vec::new();
    }
    layout.0.into_iter().filter_map(|(name, rect)| Some(Output { name: name?, rect })).collect()
}

#[derive(Default)]
struct Layout(Vec<(Option<String>, [i32; 4])>);

wayland_client::delegate_noop!(Layout: ignore wayland_client::protocol::wl_output::WlOutput);
wayland_client::delegate_noop!(Layout: wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1::ZxdgOutputManagerV1);

impl wayland_client::Dispatch<wayland_client::protocol::wl_registry::WlRegistry, wayland_client::globals::GlobalListContents> for Layout {
    fn event(
        _: &mut Self,
        _: &wayland_client::protocol::wl_registry::WlRegistry,
        _: wayland_client::protocol::wl_registry::Event,
        _: &wayland_client::globals::GlobalListContents,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl wayland_client::Dispatch<wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1::ZxdgOutputV1, usize> for Layout {
    fn event(
        layout: &mut Self,
        _: &wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1::ZxdgOutputV1,
        event: wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1::Event,
        index: &usize,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1::Event;
        let Some((name, rect)) = layout.0.get_mut(*index) else { return };
        match event {
            Event::Name { name: value } => *name = Some(value),
            Event::LogicalPosition { x, y } => (rect[0], rect[1]) = (x, y),
            Event::LogicalSize { width, height } => (rect[2], rect[3]) = (width, height),
            _ => {}
        }
    }
}
