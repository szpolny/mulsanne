mod car_render;
mod commands;
mod damage;
pub mod db;
pub mod demo;
mod engine;
pub mod live_share;
#[cfg(target_os = "linux")]
mod viewport;
#[cfg(target_os = "linux")]
mod wayland;
pub mod lmu;
pub mod model;
pub mod online;
pub mod recorder;
pub mod results;
#[cfg(test)]
mod results_e2e;
pub mod schedule;
mod shortcuts;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder, WindowEvent};

pub struct AppState {
    pub shared: Arc<engine::Shared>,
    pub db: Arc<db::Db>,
    pub config_path: PathBuf,
    pub data_dir: PathBuf,
    pub edit_mode: AtomicBool,
    pub overlay_visible: AtomicBool,
    /// Created on first use by the companion's widget previews.
    pub preview: Mutex<Option<engine::PreviewSim>>,
    /// Wakes the lap uploader after laps are queued.
    pub uploads: Arc<tokio::sync::Notify>,
    /// The open live timing share, if any.
    pub live: live_share::LiveShare,
    /// Monitor the overlay currently covers, as stored in `general.monitor` ("" = primary).
    pub overlay_monitor: Mutex<String>,
    /// Global shortcuts bound to the layout editor and overlay visibility.
    pub shortcuts: shortcuts::Shortcuts,
    /// Title bar the companion window was opened with (`general.titleBar`, Linux only).
    pub title_bar: String,
    /// GTK's window button layout, which Mulsanne's own title bar follows on Linux.
    pub decoration_layout: Option<String>,
}

/// Detect the non-composited GTK path before constructing the overlay. Keep
/// full-view invalidation off machines that can render composited buffers.
#[cfg(target_os = "linux")]
fn needs_full_repaint() -> bool {
    use gtk::prelude::*;
    if ["WEBKIT_DISABLE_DMABUF_RENDERER", "WEBKIT_DISABLE_COMPOSITING_MODE"].iter()
        .any(|key| std::env::var(key).is_ok_and(|value| value != "0")) {
        return true;
    }
    let probe = gtk::Window::new(gtk::WindowType::Popup);
    probe.realize();
    let can_composite = probe.window()
        .and_then(|window| window.create_gl_context().ok())
        .is_some_and(|context| context.realize().is_ok());
    probe.close();
    !can_composite
}

/// Transparent, click-through, always-on-top window covering the primary
/// monitor, following the game client on Linux. All widgets render inside it at their configured positions.
/// On Wayland compositors with layer shell it is a layer surface above fullscreen windows.
fn create_overlay(app: &tauri::App, monitor: &str) -> tauri::Result<()> {
    #[cfg(target_os = "linux")]
    let full_repaint = needs_full_repaint();
    #[cfg(not(target_os = "linux"))]
    let full_repaint = false;
    let window = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("index.html#overlay".into()))
        .title("Mulsanne HUD")
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .always_on_top(true)
        .visible_on_all_workspaces(false)
        .initialization_script(format!("window.__MULSANNE_FULL_REPAINT__ = {full_repaint};"))
        .skip_taskbar(true)
        // GTK ignores programmatic resize requests for non-resizable windows.
        .resizable(cfg!(target_os = "linux"))
        .focused(false)
        .visible(false)
        .build()?;
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::*;
        let gtk_window = window.gtk_window()?;
        if wayland::active() {
            wayland::init_overlay(&gtk_window);
        } else {
            // Tiling X11 window managers need a dock hint before mapping the window; an
            // always-on-top hint alone does not keep a normal client out of tiling.
            gtk_window.set_type_hint(gtk::gdk::WindowTypeHint::Dock);
        }
    }
    place_overlay(app.handle(), monitor)?;
    // GTK creates the underlying GdkWindow on show. Tao's input-shape call
    // requires that window to exist, so enabling click-through before show
    // panics on Linux.
    window.show()?;
    window.set_ignore_cursor_events(true)?;
    Ok(())
}

/// Installs a GTK header bar before mapping the companion so GTK draws and
/// themes the window controls on both Wayland and X11.
#[cfg(target_os = "linux")]
fn open_companion(app: &tauri::App, title_bar: &str) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    if title_bar == "system" {
        use gtk::gdk::prelude::GdkPixbufExt;
        use gtk::prelude::*;
        let gtk_window = window.gtk_window()?;
        let title = gtk_window.title();
        if gtk_window.is_realized() {
            gtk_window.unrealize();
        }
        let header = gtk::HeaderBar::new();
        header.set_has_subtitle(false);
        header.set_show_close_button(true);

        // Match the companion header surface. GTK styles its own title buttons.
        let css = gtk::CssProvider::new();
        css.load_from_data(b"headerbar { min-height: 43px; padding: 0 10px; background: #0b0b0e; border: 0; border-bottom: 1px solid #2a2a31; border-radius: 0; box-shadow: none; }")
            .map_err(std::io::Error::other)?;
        header.style_context().add_provider(&css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // The 4× raster of the existing companion wordmark and tagline stays
        // the same logical size on both standard- and high-DPI monitors.
        let bytes = gtk::glib::Bytes::from_static(include_bytes!("../icons/source/companion-title.png"));
        let stream = gtk::gio::MemoryInputStream::from_bytes(&bytes);
        let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_stream(&stream, gtk::gio::Cancellable::NONE)
            .map_err(std::io::Error::other)?;
        let surface = pixbuf.create_surface(4, None::<&gtk::gdk::Window>);
        let logo = gtk::Image::from_surface(surface.as_ref());
        logo.set_margin_start(6);
        // Suppress the centered GTK label while keeping the window's title
        // available to the compositor and window switcher.
        header.set_custom_title(Some(&gtk::Box::new(gtk::Orientation::Horizontal, 0)));
        header.pack_start(&logo);
        logo.show();

        gtk_window.set_titlebar(Some(&header));
        if let Some(title) = title {
            gtk_window.set_title(&title);
        }
        gtk_window.set_decorated(true);
        header.show();
    }
    window.show()
}

/// Moves the overlay to cover the monitor named `monitor` (`general.monitor`),
/// falling back to the primary monitor when it's empty or not connected.
pub(crate) fn place_overlay(app: &tauri::AppHandle, monitor: &str) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window("overlay") else { return Ok(()) };
    #[cfg(target_os = "linux")]
    if wayland::active() {
        wayland::place(&window.gtk_window()?, monitor);
        *app.state::<AppState>().overlay_monitor.lock().unwrap_or_else(|e| e.into_inner()) = monitor.to_string();
        return Ok(());
    }
    let chosen = match monitor {
        "" => None,
        name => window.available_monitors()?.into_iter().find(|m| m.name().is_some_and(|n| n == name)),
    };
    let Some(target) = chosen.or(window.primary_monitor()?) else { return Ok(()) };
    let (pos, size) = (target.position(), target.size());
    window.set_position(PhysicalPosition::new(pos.x, pos.y))?;
    // Sized after the move: crossing to a monitor with a different DPI rescales the window.
    window.set_size(PhysicalSize::new(size.width, size.height))?;
    *app.state::<AppState>().overlay_monitor.lock().unwrap_or_else(|e| e.into_inner()) = monitor.to_string();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Wayland compositors with layer shell get a native HUD; everything else
    // runs on X11, directly or through XWayland.
    #[cfg(target_os = "linux")]
    wayland::select_backend();

    // Keep the compositing renderer, but transport buffers through shared
    // memory to avoid NVIDIA/XWayland DMA-BUF allocation failures. Disabling
    // the renderer itself breaks transparent damage painting on WebKit 2.54.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DMABUF_RENDERER_FORCE_SHM").is_none() {
        std::env::set_var("WEBKIT_DMABUF_RENDERER_FORCE_SHM", "1");
    }

    tauri::Builder::default()
        .register_asynchronous_uri_scheme_protocol(car_render::SCHEME, car_render::handle)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(shortcuts::handle).build())
        .setup(move |app| {
            // Dev builds keep their own laps, identity key and config so they never touch the installed app's.
            let (mut data_dir, mut config_dir) = (app.path().app_data_dir()?, app.path().app_config_dir()?);
            // Carry data over from before the identifier changed from com.lm-overlay.app.
            for dir in [&data_dir, &config_dir] {
                let legacy = dir.with_file_name("com.lm-overlay.app");
                if !dir.exists() && legacy.is_dir() {
                    if let Err(e) = std::fs::rename(&legacy, dir) {
                        eprintln!("migrating {}: {e}", legacy.display());
                    }
                }
            }
            if cfg!(debug_assertions) {
                data_dir.push("dev");
                config_dir.push("dev");
            }
            std::fs::create_dir_all(&data_dir)?;
            std::fs::create_dir_all(&config_dir)?;

            let db = Arc::new(db::Db::open(&data_dir.join("laps.db")).map_err(|e| format!("database: {e}"))?);
            let shared = Arc::new(engine::Shared::new());
            *shared.game_dir.write().unwrap_or_else(|e| e.into_inner()) =
                car_render::load_game_dir(&data_dir).or_else(car_render::find_steam_install);
            let config_path = config_dir.join("config.json");
            let uploads = Arc::new(tokio::sync::Notify::new());

            let stored = std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
            let general = |key: &str| stored.as_ref().and_then(|c| c.pointer(&format!("/general/{key}")).cloned());
            if let Some(hz) = general("updateHz").and_then(|v| v.as_u64()) {
                shared.emit_hz.store(hz as u32, Ordering::Relaxed);
            }
            let monitor = general("monitor").and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_default();
            #[cfg(target_os = "linux")]
            let (title_bar, decoration_layout) = {
                let mode = match stored.as_ref().and_then(|c| c.pointer("/general/titleBar")).and_then(|v| v.as_str()) {
                    Some("app") => "app",
                    Some("none") => "none",
                    _ => "system",
                };
                use gtk::prelude::*;
                let layout = gtk::Settings::default().and_then(|s| s.gtk_decoration_layout()).map(|l| l.to_string());
                (mode.to_string(), layout)
            };
            #[cfg(not(target_os = "linux"))]
            let (title_bar, decoration_layout) = ("app".to_string(), None);

            app.manage(AppState {
                shared: shared.clone(),
                db: db.clone(),
                config_path,
                data_dir,
                edit_mode: AtomicBool::new(false),
                overlay_visible: AtomicBool::new(true),
                preview: Mutex::new(None),
                uploads: uploads.clone(),
                live: Default::default(),
                overlay_monitor: Mutex::new(String::new()),
                shortcuts: Default::default(),
                title_bar,
                decoration_layout,
            });

            #[cfg(target_os = "linux")]
            open_companion(app, &app.state::<AppState>().title_bar)?;
            create_overlay(app, &monitor)?;

            let shortcut = |key| general(key).and_then(|v| v.as_str().map(str::to_owned));
            shortcuts::register_initial(
                app.handle(),
                shortcut("editShortcut").as_deref(),
                shortcut("visibilityShortcut").as_deref(),
            );

            engine::spawn(app.handle().clone(), shared, db);
            online::spawn_uploader(app.handle().clone(), uploads);
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the companion app closes the overlay too.
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { .. } = event {
                    window.app_handle().exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::get_status,
            commands::latest_release,
            commands::get_config,
            commands::save_config,
            commands::set_edit_mode,
            commands::set_overlay_visible,
            commands::set_demo_mode,
            commands::set_shortcut,
            commands::list_monitors,
            commands::get_preview_snapshot,
            commands::list_sessions,
            commands::list_laps,
            commands::list_track_laps,
            commands::get_lap_trace,
            commands::personal_bests,
            commands::get_filters,
            commands::get_track_map,
            commands::delete_lap,
            commands::delete_session,
            commands::session_results,
            commands::daily_races,
            commands::online_register,
            commands::online_me,
            commands::online_set_display_name,
            commands::online_delete_data,
            commands::upload_laps,
            commands::lap_upload_states,
            commands::remove_lap_upload,
            commands::leaderboard,
            commands::leaderboard_boards,
            commands::ranked_lap_trace,
            commands::live_share_start,
            commands::live_share_stop,
            commands::live_share_status,
            commands::live_share_open,
            commands::car_liveries,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
