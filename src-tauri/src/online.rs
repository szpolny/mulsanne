//! Online features: this install's identity, signed requests to the
//! Mulsanne API, and the lap upload queue.
//!
//! The identity is an Ed25519 key in `identity.key` in the data folder. It's
//! created when the user opts in, never leaves Rust, and is kept out of
//! `config.json` so sharing a config can't leak it. Every request carries the
//! public key, a timestamp and a signature over
//! `METHOD \n path?query \n timestamp \n hex(sha256(body))`; the server
//! checks the same message (its `mulsanne-api` crate, tested against the
//! same vector as [`tests::matches_server_vector`]).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use reqwest::{Method, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;

use crate::AppState;

const API_URL: &str = "https://api.mulsanne.xyz";
const KEY_FILE: &str = "identity.key";
/// How often the uploader retries failed uploads without being woken.
const UPLOAD_POLL: Duration = Duration::from_secs(60);

/// The registered user, as returned by the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Me {
    pub id: String,
    pub display_name: Option<String>,
    /// Unix seconds.
    pub created_at: i64,
}

/// `POST /v1/laps` body. Times in seconds, distances in metres, speeds in m/s.
#[derive(Debug, Clone, Serialize)]
pub struct LapUpload {
    pub track: String,
    pub track_length: f64,
    pub car: String,
    pub vehicle_name: String,
    pub car_class: String,
    pub session_type: String,
    /// LMU build number, when known.
    pub game_version: Option<i32>,
    pub lap_time: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
    pub fuel_used: Option<f64>,
    pub ve_used: Option<f64>,
    pub max_speed: f64,
    /// Unix seconds.
    pub recorded_at: i64,
    pub app_version: String,
    pub anonymous: bool,
}

#[derive(Debug, Deserialize)]
struct LapUploaded {
    id: String,
    /// `ranked` or `held`.
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub lap_id: String,
    /// `None` for anonymous laps.
    pub driver: Option<String>,
    pub car: String,
    pub vehicle_name: String,
    pub session_type: String,
    pub game_version: Option<i32>,
    pub lap_time: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
    pub recorded_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leaderboard {
    pub entries: Vec<LeaderboardEntry>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct BoardSummary {
    pub track: String,
    pub car_class: String,
    pub drivers: i64,
    pub best_lap: f64,
    /// Game versions with ranked laps, newest first.
    pub game_versions: Vec<i32>,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
}

#[derive(Debug)]
pub enum RequestError {
    /// No response: offline, DNS, timeout. Worth retrying.
    Network(String),
    Status(StatusCode, String),
    Other(String),
}

impl RequestError {
    /// The server refused the request itself; sending it again won't help.
    fn is_rejection(&self) -> bool {
        matches!(self, RequestError::Status(s, _)
            if s.is_client_error() && *s != StatusCode::REQUEST_TIMEOUT && *s != StatusCode::TOO_MANY_REQUESTS)
    }
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::Network(m) | RequestError::Status(_, m) | RequestError::Other(m) => f.write_str(m),
        }
    }
}

impl From<RequestError> for String {
    fn from(e: RequestError) -> Self {
        e.to_string()
    }
}

/// Online settings from `config.json` (the frontend owns the schema).
#[derive(Debug, Clone, Copy, Default)]
pub struct Settings {
    pub enabled: bool,
    /// Upload every eligible lap as it's recorded.
    pub auto_upload: bool,
    pub anonymous: bool,
}

pub fn settings(config_path: &Path) -> Settings {
    let Some(config) = std::fs::read_to_string(config_path).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok())
    else {
        return Settings::default();
    };
    let flag = |pointer| config.pointer(pointer).and_then(Value::as_bool).unwrap_or(false);
    Settings {
        enabled: flag("/online/enabled"),
        auto_upload: config.pointer("/online/lapUpload").and_then(Value::as_str) == Some("auto"),
        anonymous: flag("/online/anonymous"),
    }
}

pub fn require_enabled(config_path: &Path) -> Result<Settings, String> {
    let s = settings(config_path);
    if s.enabled {
        Ok(s)
    } else {
        Err("Online features are turned off".into())
    }
}

/// `MULSANNE_API_URL` overrides the server in dev builds only.
fn api_url() -> String {
    if cfg!(debug_assertions) {
        if let Ok(url) = std::env::var("MULSANNE_API_URL") {
            return url.trim_end_matches('/').to_owned();
        }
    }
    API_URL.to_owned()
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn key_path(data_dir: &Path) -> PathBuf {
    data_dir.join(KEY_FILE)
}

pub fn load_key(data_dir: &Path) -> Result<Option<SigningKey>, String> {
    let text = match std::fs::read_to_string(key_path(data_dir)) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Could not read the online identity: {e}")),
    };
    let bytes: [u8; 32] = URL_SAFE_NO_PAD
        .decode(text.trim())
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or("The online identity file is damaged")?;
    Ok(Some(SigningKey::from_bytes(&bytes)))
}

pub fn load_or_create_key(data_dir: &Path) -> Result<SigningKey, String> {
    if let Some(key) = load_key(data_dir)? {
        return Ok(key);
    }
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).map_err(|e| format!("Could not generate an online identity: {e}"))?;
    let key = SigningKey::from_bytes(&seed);
    // Write then rename so a crash can't leave a truncated key behind.
    let tmp = data_dir.join(format!("{KEY_FILE}.tmp"));
    std::fs::write(&tmp, URL_SAFE_NO_PAD.encode(seed))
        .and_then(|_| std::fs::rename(&tmp, key_path(data_dir)))
        .map_err(|e| format!("Could not save the online identity: {e}"))?;
    Ok(key)
}

pub fn delete_key(data_dir: &Path) -> Result<(), String> {
    match std::fs::remove_file(key_path(data_dir)) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(format!("Could not delete the online identity: {e}")),
        _ => Ok(()),
    }
}

fn message(method: &str, path_and_query: &str, timestamp: i64, body: &[u8]) -> String {
    let hex: String = Sha256::digest(body).iter().map(|b| format!("{b:02x}")).collect();
    format!("{}\n{}\n{}\n{}", method.to_ascii_uppercase(), path_and_query, timestamp, hex)
}

/// Header values for a signed request: (key, timestamp, signature).
fn sign(key: &SigningKey, method: &str, path_and_query: &str, timestamp: i64, body: &[u8]) -> (String, String, String) {
    let sig = key.sign(message(method, path_and_query, timestamp, body).as_bytes());
    (
        URL_SAFE_NO_PAD.encode(key.verifying_key().as_bytes()),
        timestamp.to_string(),
        URL_SAFE_NO_PAD.encode(sig.to_bytes()),
    )
}

fn client() -> Result<reqwest::Client, RequestError> {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    reqwest::Client::builder()
        .user_agent(concat!("Mulsanne/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| RequestError::Other(e.to_string()))
}

/// Sends a request, signed when `key` is given. Returns `None` for an empty
/// (204) response.
async fn request<T: DeserializeOwned>(
    key: Option<&SigningKey>,
    method: Method,
    path_and_query: &str,
    body: Option<&Value>,
) -> Result<Option<T>, RequestError> {
    let body = body.map(Value::to_string).unwrap_or_default();
    let mut req = client()?
        .request(method.clone(), format!("{}{path_and_query}", api_url()))
        .header("content-type", "application/json");
    if let Some(key) = key {
        let (k, t, s) = sign(key, method.as_str(), path_and_query, now(), body.as_bytes());
        req = req.header("x-mulsanne-key", k).header("x-mulsanne-timestamp", t).header("x-mulsanne-signature", s);
    }
    let res = req
        .body(body)
        .send()
        .await
        .map_err(|e| RequestError::Network(format!("Could not reach the Mulsanne server: {e}")))?;

    let status = res.status();
    let bytes = res.bytes().await.map_err(|e| RequestError::Network(e.to_string()))?;
    if !status.is_success() {
        let message = serde_json::from_slice::<ApiError>(&bytes)
            .map(|e| e.message)
            .unwrap_or_else(|_| format!("The Mulsanne server returned {status}"));
        return Err(RequestError::Status(status, message));
    }
    if bytes.is_empty() {
        return Ok(None);
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| RequestError::Other(format!("Unexpected server response: {e}")))
}

async fn request_some<T: DeserializeOwned>(
    key: Option<&SigningKey>,
    method: Method,
    path_and_query: &str,
    body: Option<&Value>,
) -> Result<T, RequestError> {
    request(key, method, path_and_query, body)
        .await?
        .ok_or_else(|| RequestError::Other("Unexpected empty server response".into()))
}

/// Creates the identity if needed and registers it. Registering an existing
/// identity returns the same user.
pub async fn register(data_dir: &Path, display_name: Option<String>) -> Result<Me, String> {
    let key = load_or_create_key(data_dir)?;
    let body = serde_json::json!({ "display_name": display_name });
    Ok(request_some(Some(&key), Method::POST, "/v1/users", Some(&body)).await?)
}

fn require_key(data_dir: &Path) -> Result<SigningKey, String> {
    load_key(data_dir)?.ok_or_else(|| "Online features are not set up on this PC".into())
}

pub async fn me(data_dir: &Path) -> Result<Me, String> {
    Ok(request_some(Some(&require_key(data_dir)?), Method::GET, "/v1/me", None).await?)
}

pub async fn set_display_name(data_dir: &Path, display_name: Option<String>) -> Result<Me, String> {
    let body = serde_json::json!({ "display_name": display_name });
    Ok(request_some(Some(&require_key(data_dir)?), Method::PATCH, "/v1/me", Some(&body)).await?)
}

/// Deletes the user and everything they uploaded, then the local identity.
pub async fn delete_data(data_dir: &Path) -> Result<(), String> {
    if let Some(key) = load_key(data_dir)? {
        request::<Value>(Some(&key), Method::DELETE, "/v1/me", None).await?;
    }
    delete_key(data_dir)
}

/// Removes an uploaded lap from the rankings.
pub async fn delete_lap(data_dir: &Path, remote_id: &str) -> Result<(), String> {
    let key = require_key(data_dir)?;
    match request::<Value>(Some(&key), Method::DELETE, &format!("/v1/laps/{remote_id}"), None).await {
        // Already gone on the server.
        Ok(_) | Err(RequestError::Status(StatusCode::NOT_FOUND, _)) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Leaderboard filters, as the frontend sends them.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardQuery {
    pub track: String,
    pub car_class: String,
    pub car: Option<String>,
    pub session_type: Option<String>,
    pub game_version: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn leaderboard(q: &LeaderboardQuery) -> Result<Leaderboard, String> {
    let mut params = vec![
        ("track", q.track.clone()),
        ("car_class", q.car_class.clone()),
        ("limit", q.limit.unwrap_or(50).to_string()),
        ("offset", q.offset.unwrap_or(0).to_string()),
    ];
    params.extend(q.car.clone().map(|c| ("car", c)));
    params.extend(q.session_type.clone().map(|s| ("session_type", s)));
    params.extend(q.game_version.map(|v| ("game_version", v.to_string())));
    let url = Url::parse_with_params("http://x/v1/leaderboards", &params).map_err(|e| e.to_string())?;
    let path = format!("{}?{}", url.path(), url.query().unwrap_or(""));
    Ok(request_some(None, Method::GET, &path, None).await?)
}

pub async fn boards() -> Result<Vec<BoardSummary>, String> {
    Ok(request_some(None, Method::GET, "/v1/leaderboards/boards", None).await?)
}

/// Uploads queued laps in the background. Woken by `uploads.notify_one()`
/// and every [`UPLOAD_POLL`] to retry failures; emits `lap-uploads-changed`
/// after each attempt.
pub fn spawn_uploader(app: AppHandle, wake: Arc<Notify>) {
    tauri::async_runtime::spawn(async move {
        loop {
            drain_uploads(&app).await;
            tokio::select! {
                _ = wake.notified() => {}
                _ = tokio::time::sleep(UPLOAD_POLL) => {}
            }
        }
    });
}

async fn drain_uploads(app: &AppHandle) {
    let state = app.state::<AppState>();
    if !settings(&state.config_path).enabled {
        return;
    }
    let Ok(Some(key)) = load_key(&state.data_dir) else { return };
    let version = app.package_info().version.to_string();
    loop {
        let next = match state.db.next_upload(now(), &version) {
            Ok(Some(next)) => next,
            Ok(None) => return,
            Err(e) => {
                eprintln!("upload queue: {e}");
                return;
            }
        };
        let body = serde_json::to_value(&next.lap).unwrap_or(Value::Null);
        let result = request_some::<LapUploaded>(Some(&key), Method::POST, "/v1/laps", Some(&body)).await;
        // Over the server's upload limit: keep it queued (without using up a
        // retry) and try again on the next poll.
        if matches!(result, Err(RequestError::Status(StatusCode::TOO_MANY_REQUESTS, _))) {
            return;
        }
        let saved = match &result {
            Ok(r) => state.db.set_upload_status(next.lap_id, &r.status, Some(&r.id), None, now()),
            Err(e) if e.is_rejection() => state.db.set_upload_status(next.lap_id, "rejected", None, Some(&e.to_string()), now()),
            Err(e) => state.db.set_upload_status(next.lap_id, "failed", None, Some(&e.to_string()), now()),
        };
        if let Err(e) = saved {
            eprintln!("upload queue: {e}");
            return;
        }
        let _ = app.emit("lap-uploads-changed", next.lap_id);
        // Offline or a server problem: leave the rest queued for the next poll.
        if result.as_ref().is_err_and(|e| !e.is_rejection()) {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Same vector as `signing::tests::matches_app_vector` in mulsanne-server.
    #[test]
    fn matches_server_vector() {
        let key = SigningKey::from_bytes(&[7; 32]);
        let (k, t, s) = sign(&key, "POST", "/v1/users", 1_700_000_000, br#"{"display_name":"Test"}"#);
        assert_eq!(k, "6kpsY-KcUgq-9VB7Ey7F-ZVHdq6-vnuSQh7qaRRG0iw");
        assert_eq!(t, "1700000000");
        assert_eq!(s, "0QCsbQqDOdai67gF5wVmmMC0_l_wVweKB6A8Uhp82USV2RCAXWABI-wQBIX9c9m4uE4RIxTceZb0xEbUI54sBw");
    }

    #[test]
    fn key_file_lifecycle() {
        let dir = std::env::temp_dir().join(format!("mulsanne-online-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(load_key(&dir).unwrap().is_none());
        let created = load_or_create_key(&dir).unwrap();
        let loaded = load_or_create_key(&dir).unwrap();
        assert_eq!(created.to_bytes(), loaded.to_bytes());
        delete_key(&dir).unwrap();
        delete_key(&dir).unwrap();
        assert!(load_key(&dir).unwrap().is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn damaged_key_file_is_an_error() {
        let dir = std::env::temp_dir().join(format!("mulsanne-online-damaged-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(key_path(&dir), "not a key").unwrap();
        assert!(load_key(&dir).is_err());
        // Never silently replaced: that would orphan the user's online data.
        assert!(load_or_create_key(&dir).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn reads_settings_from_config() {
        let dir = std::env::temp_dir().join(format!("mulsanne-online-settings-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        assert!(!settings(&path).enabled);
        std::fs::write(&path, r#"{"online":{"enabled":true,"lapUpload":"auto","anonymous":true}}"#).unwrap();
        let s = settings(&path);
        assert!(s.enabled && s.auto_upload && s.anonymous);
        std::fs::write(&path, r#"{"online":{"enabled":true,"lapUpload":"manual"}}"#).unwrap();
        let s = settings(&path);
        assert!(s.enabled && !s.auto_upload && !s.anonymous);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn classifies_errors() {
        assert!(RequestError::Status(StatusCode::UNPROCESSABLE_ENTITY, String::new()).is_rejection());
        assert!(!RequestError::Status(StatusCode::TOO_MANY_REQUESTS, String::new()).is_rejection());
        assert!(!RequestError::Status(StatusCode::BAD_GATEWAY, String::new()).is_rejection());
        assert!(!RequestError::Network(String::new()).is_rejection());
    }
}
