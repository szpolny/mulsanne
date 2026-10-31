mod commands;
mod damage;
pub mod db;
pub mod demo;
mod engine;
pub mod lmu;
pub mod model;
pub mod online;
pub mod recorder;
pub mod schedule;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

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
}

/// Transparent, click-through, always-on-top window covering the primary
/// monitor. All widgets render inside it at their configured positions.
/// Works over LMU in borderless/windowed mode (not exclusive fullscreen).
fn create_overlay(app: &tauri::App) -> tauri::Result<()> {
    let window = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("index.html#overlay".into()))
        .title("Mulsanne HUD")
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .visible(false)
        .build()?;
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::GtkWindowExt;
        // Tiling compositors need a dock hint before mapping the window; an
        // always-on-top hint alone does not keep a normal client out of tiling.
        window.gtk_window()?.set_type_hint(gtk::gdk::WindowTypeHint::Dock);
    }
    if let Some(monitor) = app.primary_monitor()? {
        window.set_position(PhysicalPosition::new(monitor.position().x, monitor.position().y))?;
        window.set_size(PhysicalSize::new(monitor.size().width, monitor.size().height))?;
    }
    // GTK creates the underlying GdkWindow on show. Tao's input-shape call
    // requires that window to exist, so enabling click-through before show
    // panics on Linux.
    window.show()?;
    window.set_ignore_cursor_events(true)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Overlay placement, stacking, click-through, and global shortcuts use X11.
    // Wayland sessions can run the app through XWayland; select it before GTK
    // initializes rather than silently creating an unsupported native surface.
    #[cfg(target_os = "linux")]
    if std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    // WebKit's DMA-BUF path can fail to allocate buffers on NVIDIA/XWayland.
    // The fallback still supports transparent webviews and avoids a blank HUD.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let edit_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::F9);
    let visibility_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::F10);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let state = app.state::<AppState>();
                    if shortcut == &edit_shortcut {
                        let enabled = !state.edit_mode.load(Ordering::Acquire);
                        commands::apply_edit_mode(app, enabled);
                    } else if shortcut == &visibility_shortcut {
                        let visible = !state.overlay_visible.load(Ordering::Acquire);
                        commands::apply_overlay_visible(app, visible);
                    }
                })
                .build(),
        )
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
            let config_path = config_dir.join("config.json");
            let uploads = Arc::new(tokio::sync::Notify::new());

            if let Some(hz) = std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|c| c.pointer("/general/updateHz").and_then(|v| v.as_u64()))
            {
                shared.emit_hz.store(hz as u32, Ordering::Relaxed);
            }

            app.manage(AppState {
                shared: shared.clone(),
                db: db.clone(),
                config_path,
                data_dir,
                edit_mode: AtomicBool::new(false),
                overlay_visible: AtomicBool::new(true),
                preview: Mutex::new(None),
                uploads: uploads.clone(),
            });

            create_overlay(app)?;

            let shortcuts = app.global_shortcut();
            for sc in [edit_shortcut, visibility_shortcut] {
                if let Err(e) = shortcuts.register(sc) {
                    eprintln!("could not register shortcut {sc:?}: {e}");
                }
            }

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
            commands::get_config,
            commands::save_config,
            commands::set_edit_mode,
            commands::set_overlay_visible,
            commands::set_demo_mode,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
