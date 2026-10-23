//! Online features: this install's identity and signed requests to the
//! Mulsanne API.
//!
//! The identity is an Ed25519 key in `identity.key` in the data folder. It's
//! created when the user opts in, never leaves Rust, and is kept out of
//! `config.json` so sharing a config can't leak it. Every request carries the
//! public key, a timestamp and a signature over
//! `METHOD \n path?query \n timestamp \n hex(sha256(body))`; the server
//! checks the same message (its `mulsanne-api` crate, tested against the
//! same vector as [`tests::matches_server_vector`]).

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

const API_URL: &str = "https://api.mulsanne.xyz";
const KEY_FILE: &str = "identity.key";

/// The registered user, as returned by the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Me {
    pub id: String,
    pub display_name: Option<String>,
    /// Unix seconds.
    pub created_at: i64,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
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

/// Sends a signed request. Returns `None` for an empty (204) response.
async fn request<T: DeserializeOwned>(
    key: &SigningKey,
    method: Method,
    path: &str,
    body: Option<&Value>,
) -> Result<Option<T>, String> {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    let body = body.map(Value::to_string).unwrap_or_default();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (k, t, s) = sign(key, method.as_str(), path, timestamp, body.as_bytes());

    let client = reqwest::Client::builder()
        .user_agent(concat!("Mulsanne/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .request(method, format!("{}{path}", api_url()))
        .header("content-type", "application/json")
        .header("x-mulsanne-key", k)
        .header("x-mulsanne-timestamp", t)
        .header("x-mulsanne-signature", s)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Could not reach the Mulsanne server: {e}"))?;

    let status = res.status();
    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(serde_json::from_slice::<ApiError>(&bytes)
            .map(|e| e.message)
            .unwrap_or_else(|_| format!("The Mulsanne server returned {status}")));
    }
    if bytes.is_empty() {
        return Ok(None);
    }
    serde_json::from_slice(&bytes).map(Some).map_err(|e| format!("Unexpected server response: {e}"))
}

async fn request_me(key: &SigningKey, method: Method, path: &str, body: Option<&Value>) -> Result<Me, String> {
    request(key, method, path, body).await?.ok_or_else(|| "Unexpected empty server response".into())
}

/// Creates the identity if needed and registers it. Registering an existing
/// identity returns the same user.
pub async fn register(data_dir: &Path, display_name: Option<String>) -> Result<Me, String> {
    let key = load_or_create_key(data_dir)?;
    request_me(&key, Method::POST, "/v1/users", Some(&serde_json::json!({ "display_name": display_name }))).await
}

fn require_key(data_dir: &Path) -> Result<SigningKey, String> {
    load_key(data_dir)?.ok_or_else(|| "Online features are not set up on this PC".into())
}

pub async fn me(data_dir: &Path) -> Result<Me, String> {
    request_me(&require_key(data_dir)?, Method::GET, "/v1/me", None).await
}

pub async fn set_display_name(data_dir: &Path, display_name: Option<String>) -> Result<Me, String> {
    let body = serde_json::json!({ "display_name": display_name });
    request_me(&require_key(data_dir)?, Method::PATCH, "/v1/me", Some(&body)).await
}

/// Deletes the user and everything they uploaded, then the local identity.
pub async fn delete_data(data_dir: &Path) -> Result<(), String> {
    if let Some(key) = load_key(data_dir)? {
        request::<Value>(&key, Method::DELETE, "/v1/me", None).await?;
    }
    delete_key(data_dir)
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
}
