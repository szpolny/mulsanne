//! Serves the car renders LMU ships in `Bin/UI.zip`, one set per livery, over
//! a custom URI scheme: `car-render://localhost/<view>.<veh file>`.
//!
//! They are the game's assets, so they're read from the player's own install
//! and never bundled with the app.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use tauri::http::{header, Request, Response, StatusCode};
use tauri::{AppHandle, Manager, Runtime, UriSchemeContext, UriSchemeResponder};

use crate::engine::Shared;
use crate::lmu::shm;
use crate::AppState;

pub const SCHEME: &str = "car-render";

/// Remembers the install folder so renders still load when the game is closed.
const GAME_DIR_FILE: &str = "game-dir.txt";

pub fn load_game_dir(data_dir: &Path) -> Option<PathBuf> {
    let dir = std::fs::read_to_string(data_dir.join(GAME_DIR_FILE)).ok()?;
    Some(PathBuf::from(dir.trim())).filter(|d| !d.as_os_str().is_empty())
}

pub fn remember_game_dir(app: &AppHandle, shared: &Shared, hwnd: u64) {
    let Some(dir) = shm::process_dir(hwnd) else { return };
    if let Some(state) = app.try_state::<AppState>() {
        if let Err(e) = std::fs::write(state.data_dir.join(GAME_DIR_FILE), dir.to_string_lossy().as_bytes()) {
            eprintln!("saving game folder: {e}");
        }
    }
    *shared.game_dir.write().unwrap_or_else(|e| e.into_inner()) = Some(dir);
}

/// The archive entry for a request path such as `/front.397_25_963`.
fn entry_name(path: &str) -> Option<String> {
    let (view, veh) = path.trim_start_matches('/').split_once('.')?;
    if veh.is_empty() || !veh.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return None;
    }
    let (folder, suffix) = match view {
        "front" => ("FrontLarge", "frontAngle"),
        "thumb" => ("FrontThumbnail", "frontAngle"),
        "top" => ("topDownRotated", "topDown"),
        _ => return None,
    };
    Some(format!("start\\images\\cars\\{folder}\\{veh}_{suffix}.webp"))
}

fn read_entry(archive_path: &Path, entry: &str) -> Option<Vec<u8>> {
    // Opened per request: keeping it open would stop Steam replacing the file in a game update.
    let file = File::open(archive_path).ok()?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file)).ok()?;
    let mut image = archive.by_name(entry).ok()?;
    let mut bytes = Vec::with_capacity(image.size() as usize);
    image.read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

fn respond(bytes: Option<Vec<u8>>) -> Response<Vec<u8>> {
    let builder = Response::builder();
    let response = match bytes {
        Some(bytes) => builder
            .header(header::CONTENT_TYPE, "image/webp")
            .header(header::CACHE_CONTROL, "public, max-age=86400")
            .body(bytes),
        // Not cached: the game folder may only become known later.
        None => builder.status(StatusCode::NOT_FOUND).body(Vec::new()),
    };
    response.expect("static response parts are valid")
}

pub fn handle<R: Runtime>(ctx: UriSchemeContext<'_, R>, request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let app = ctx.app_handle().clone();
    let path = request.uri().path().to_string();
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = entry_name(&path).and_then(|entry| {
            let state = app.try_state::<AppState>()?;
            let dir = state.shared.game_dir.read().unwrap_or_else(|e| e.into_inner()).clone()?;
            read_entry(&dir.join("Bin").join("UI.zip"), &entry)
        });
        responder.respond(respond(bytes));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn maps_views_to_archive_entries() {
        assert_eq!(
            entry_name("/front.23_26_THOR59931582").as_deref(),
            Some("start\\images\\cars\\FrontLarge\\23_26_THOR59931582_frontAngle.webp")
        );
        assert_eq!(
            entry_name("thumb.397_25_963").as_deref(),
            Some("start\\images\\cars\\FrontThumbnail\\397_25_963_frontAngle.webp")
        );
        assert_eq!(
            entry_name("/top.397_25_963").as_deref(),
            Some("start\\images\\cars\\topDownRotated\\397_25_963_topDown.webp")
        );
    }

    #[test]
    fn rejects_anything_but_a_known_view_and_plain_name() {
        for path in ["", "/front", "/front.", "/side.397_25_963", "/front.a/b", "/front.a\\b", "/front.a.b", "/front...%2Fx"] {
            assert_eq!(entry_name(path), None, "{path}");
        }
    }

    #[test]
    fn reads_an_entry_whose_name_uses_backslashes() {
        let path = std::env::temp_dir().join(format!("mulsanne-car-render-{}.zip", std::process::id()));
        let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
        let entry = entry_name("/top.CAR").unwrap();
        zip.start_file(entry.as_str(), zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(b"webp bytes").unwrap();
        zip.finish().unwrap();

        assert_eq!(read_entry(&path, &entry).as_deref(), Some(&b"webp bytes"[..]));
        assert_eq!(read_entry(&path, "start\\missing.webp"), None);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn missing_game_dir_file_means_unknown() {
        assert_eq!(load_game_dir(Path::new("/nonexistent/mulsanne")), None);
    }
}
