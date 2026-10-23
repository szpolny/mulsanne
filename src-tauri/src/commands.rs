use std::sync::atomic::Ordering;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::{Filters, LapRecord, LapTrace, PersonalBest, SessionSummary, TrackMap, UploadState};
use crate::engine::PreviewSim;
use crate::model::Snapshot;
use crate::online::{BoardSummary, Leaderboard, LeaderboardQuery, Me};
use crate::schedule::DailyRace;
use crate::AppState;

type CmdResult<T> = Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    demo: bool,
    edit_mode: bool,
    overlay_visible: bool,
    game_focused: bool,
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
        game_focused: state.shared.game_focused.load(Ordering::Acquire),
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

/// Next frame of the companion's preview-only demo data (see `PreviewSim`).
#[tauri::command]
pub async fn get_preview_snapshot(state: State<'_, AppState>) -> CmdResult<Snapshot> {
    let mut preview = state.preview.lock().unwrap_or_else(|e| e.into_inner());
    Ok(preview.get_or_insert_with(|| PreviewSim::new(state.db.clone())).next())
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

#[tauri::command]
pub async fn daily_races() -> CmdResult<Vec<DailyRace>> {
    crate::schedule::fetch_daily_races().await
}

#[tauri::command]
pub async fn online_register(state: State<'_, AppState>, display_name: Option<String>) -> CmdResult<Me> {
    crate::online::register(&state.data_dir, display_name).await
}

#[tauri::command]
pub async fn online_me(state: State<'_, AppState>) -> CmdResult<Me> {
    crate::online::me(&state.data_dir).await
}

#[tauri::command]
pub async fn online_set_display_name(state: State<'_, AppState>, display_name: Option<String>) -> CmdResult<Me> {
    crate::online::set_display_name(&state.data_dir, display_name).await
}

#[tauri::command]
pub async fn online_delete_data(app: AppHandle, state: State<'_, AppState>) -> CmdResult<()> {
    crate::online::delete_data(&state.data_dir).await?;
    state.db.clear_uploads()?;
    let _ = app.emit("lap-uploads-changed", ());
    Ok(())
}

/// Queues laps for the online rankings. Returns how many qualified.
#[tauri::command]
pub fn upload_laps(app: AppHandle, state: State<AppState>, lap_ids: Vec<i64>) -> CmdResult<usize> {
    let settings = crate::online::require_enabled(&state.config_path)?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let queued = state.db.enqueue_uploads(&lap_ids, settings.anonymous, now)?;
    if queued > 0 {
        state.uploads.notify_one();
        let _ = app.emit("lap-uploads-changed", ());
    }
    Ok(queued)
}

#[tauri::command]
pub fn lap_upload_states(state: State<AppState>) -> CmdResult<Vec<UploadState>> {
    state.db.upload_states()
}

/// Removes a lap from the online rankings (and forgets a queued upload).
#[tauri::command]
pub async fn remove_lap_upload(app: AppHandle, state: State<'_, AppState>, lap_id: i64) -> CmdResult<()> {
    if let Some(remote_id) = state.db.upload_remote_id(lap_id)? {
        crate::online::delete_lap(&state.data_dir, &remote_id).await?;
    }
    state.db.remove_upload(lap_id)?;
    let _ = app.emit("lap-uploads-changed", lap_id);
    Ok(())
}

#[tauri::command]
pub async fn leaderboard(state: State<'_, AppState>, query: LeaderboardQuery) -> CmdResult<Leaderboard> {
    crate::online::require_enabled(&state.config_path)?;
    crate::online::leaderboard(&query).await
}

#[tauri::command]
pub async fn leaderboard_boards(state: State<'_, AppState>) -> CmdResult<Vec<BoardSummary>> {
    crate::online::require_enabled(&state.config_path)?;
    crate::online::boards().await
}
