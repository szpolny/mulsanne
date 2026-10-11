use std::sync::atomic::Ordering;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::{Filters, LapRecord, LapTrace, PersonalBest, SessionSummary, TrackMap};
use crate::model::Snapshot;
use crate::AppState;

type CmdResult<T> = Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    demo: bool,
    edit_mode: bool,
    overlay_visible: bool,
    data_dir: String,
}

#[tauri::command]
pub fn get_snapshot(state: State<AppState>) -> Snapshot {
    state.shared.latest.read().map(|s| s.clone()).unwrap_or_default()
}

#[tauri::command]
pub fn get_status(state: State<AppState>) -> Status {
    Status {
        demo: state.shared.demo.load(Ordering::Acquire),
        edit_mode: state.edit_mode.load(Ordering::Acquire),
        overlay_visible: state.overlay_visible.load(Ordering::Acquire),
        data_dir: state.data_dir.display().to_string(),
    }
}

/// Layout and widget settings. The frontend owns the schema; the backend
/// persists it as opaque JSON.
#[tauri::command]
pub fn get_config(state: State<AppState>) -> Value {
    std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null)
}

#[tauri::command]
pub fn save_config(app: AppHandle, state: State<AppState>, config: Value) -> CmdResult<()> {
    let text = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&state.config_path, text).map_err(|e| e.to_string())?;
    if let Some(hz) = config.pointer("/general/updateHz").and_then(Value::as_u64) {
        state.shared.emit_hz.store(hz as u32, Ordering::Relaxed);
    }
    app.emit("config-changed", &config).map_err(|e| e.to_string())
}

pub fn apply_edit_mode(app: &AppHandle, enabled: bool) {
    let state = app.state::<AppState>();
    state.edit_mode.store(enabled, Ordering::Release);
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.set_ignore_cursor_events(!enabled);
        if enabled {
            let _ = overlay.show();
            let _ = overlay.set_focus();
        }
    }
    let _ = app.emit("edit-mode", enabled);
}

pub fn apply_overlay_visible(app: &AppHandle, visible: bool) {
    let state = app.state::<AppState>();
    state.overlay_visible.store(visible, Ordering::Release);
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = if visible { overlay.show() } else { overlay.hide() };
    }
    let _ = app.emit("overlay-visible", visible);
}

#[tauri::command]
pub fn set_edit_mode(app: AppHandle, enabled: bool) {
    apply_edit_mode(&app, enabled);
}

#[tauri::command]
pub fn set_overlay_visible(app: AppHandle, visible: bool) {
    apply_overlay_visible(&app, visible);
}

#[tauri::command]
pub fn set_demo_mode(app: AppHandle, state: State<AppState>, enabled: bool) {
    state.shared.demo.store(enabled, Ordering::Release);
    let _ = app.emit("demo-mode", enabled);
}

#[tauri::command]
pub fn list_sessions(
    state: State<AppState>,
    track: Option<String>,
    car: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> CmdResult<Vec<SessionSummary>> {
    state.db.list_sessions(track.as_deref(), car.as_deref(), limit.unwrap_or(100), offset.unwrap_or(0))
}

#[tauri::command]
pub fn list_laps(state: State<AppState>, session_id: i64) -> CmdResult<Vec<LapRecord>> {
    state.db.list_laps(session_id)
}

#[tauri::command]
pub fn list_track_laps(
    state: State<AppState>,
    track: String,
    car: Option<String>,
    valid_only: Option<bool>,
    limit: Option<i64>,
) -> CmdResult<Vec<LapRecord>> {
    state.db.list_track_laps(&track, car.as_deref(), valid_only.unwrap_or(false), limit.unwrap_or(200))
}

#[tauri::command]
pub fn get_lap_trace(state: State<AppState>, lap_id: i64) -> CmdResult<LapTrace> {
    state.db.lap_trace(lap_id)
}

#[tauri::command]
pub fn personal_bests(state: State<AppState>) -> CmdResult<Vec<PersonalBest>> {
    state.db.personal_bests()
}

#[tauri::command]
pub fn get_filters(state: State<AppState>) -> CmdResult<Filters> {
    state.db.filters()
}

#[tauri::command]
pub fn get_track_map(state: State<AppState>, track: String) -> CmdResult<Option<TrackMap>> {
    state.db.track_map(&track)
}

#[tauri::command]
pub fn delete_lap(state: State<AppState>, lap_id: i64) -> CmdResult<()> {
    state.db.delete_lap(lap_id)?;
    state.shared.reload_reference.store(true, Ordering::Release);
    Ok(())
}

#[tauri::command]
pub fn delete_session(state: State<AppState>, session_id: i64) -> CmdResult<()> {
    state.db.delete_session(session_id)?;
    state.shared.reload_reference.store(true, Ordering::Release);
    Ok(())
}
