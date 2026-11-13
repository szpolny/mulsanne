use std::sync::atomic::Ordering;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::db::{Filters, LapRecord, LapTrace, PersonalBest, SessionSummary, TrackMap, UploadState};
use crate::engine::PreviewSim;
use crate::live_share::ShareStatus;
use crate::model::Snapshot;
use crate::online::{BoardSummary, Leaderboard, LeaderboardQuery, Me};
use crate::schedule::Schedule;
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
    experimental_linux: bool,
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
        experimental_linux: cfg!(target_os = "linux"),
    }
}

#[derive(serde::Serialize)]
pub struct LatestRelease {
    version: String,
    notes: String,
}

/// Latest published GitHub release. Used where the app cannot update itself
/// (Linux package managers) to tell the user a newer version exists.
#[tauri::command]
pub async fn latest_release() -> Result<LatestRelease, String> {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    let client = reqwest::Client::builder()
        .user_agent(concat!("Mulsanne/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let text = client
        .get("https://api.github.com/repos/szpolny/mulsanne/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Could not reach GitHub: {e}"))?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let version = json["tag_name"]
        .as_str()
        .map(|t| t.trim_start_matches('v').to_owned())
        .filter(|v| !v.is_empty())
        .ok_or("The latest release has no version")?;
    Ok(LatestRelease { version, notes: json["body"].as_str().unwrap_or_default().to_owned() })
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
    let monitor = config.pointer("/general/monitor").and_then(Value::as_str).unwrap_or_default();
    let moved = *state.overlay_monitor.lock().unwrap_or_else(|e| e.into_inner()) != monitor;
    if moved {
        crate::place_overlay(&app, monitor).map_err(|e| e.to_string())?;
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    /// Id stored in `general.monitor`.
    name: String,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    primary: bool,
}

/// Connected monitors the overlay can be placed on.
#[tauri::command]
pub fn list_monitors(app: AppHandle) -> CmdResult<Vec<MonitorInfo>> {
    let primary = app.primary_monitor().map_err(|e| e.to_string())?.and_then(|m| m.name().cloned());
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    Ok(monitors
        .into_iter()
        .filter_map(|m| {
            let name = m.name()?.clone();
            Some(MonitorInfo {
                primary: primary.as_ref() == Some(&name),
                width: m.size().width,
                height: m.size().height,
                x: m.position().x,
                y: m.position().y,
                name,
            })
        })
        .collect())
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
pub async fn daily_races(state: State<'_, AppState>, force: bool) -> CmdResult<Schedule> {
    crate::schedule::daily_races(&state.data_dir, force).await
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
    crate::live_share::stop(&app).await;
    crate::online::delete_data(&state.data_dir).await?;
    state.db.clear_uploads()?;
    let _ = app.emit("lap-uploads-changed", ());
    Ok(())
}

/// Starts sharing live timing; returns the viewer link.
#[tauri::command]
pub async fn live_share_start(app: AppHandle) -> CmdResult<ShareStatus> {
    crate::live_share::start(&app).await
}

#[tauri::command]
pub async fn live_share_stop(app: AppHandle) -> CmdResult<()> {
    crate::live_share::stop(&app).await;
    Ok(())
}

#[tauri::command]
pub fn live_share_status(state: State<AppState>) -> Option<ShareStatus> {
    state.live.status()
}

/// Opens the open share's viewer page in the default browser. The URL comes
/// from Rust, so the webview needs no opener permission for it.
#[tauri::command]
pub fn live_share_open(app: AppHandle, state: State<AppState>) -> CmdResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let status = state.live.status().ok_or("Not sharing")?;
    app.opener().open_url(status.url, None::<&str>).map_err(|e| e.to_string())
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

/// Telemetry of a ranked lap from the online leaderboards.
#[tauri::command]
pub async fn ranked_lap_trace(state: State<'_, AppState>, remote_id: String) -> CmdResult<LapTrace> {
    crate::online::require_enabled(&state.config_path)?;
    crate::online::ranked_lap_trace(&remote_id).await
}
