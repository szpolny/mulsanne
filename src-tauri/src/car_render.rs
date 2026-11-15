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

/// Where Steam puts LMU inside a library folder.
const STEAM_GAME_DIR: &str = "steamapps/common/Le Mans Ultimate";

/// Finds LMU in the player's Steam libraries, so renders load before the game
/// has ever run alongside the app.
pub fn find_steam_install() -> Option<PathBuf> {
    find_in_steam(&steam_roots())
}

fn find_in_steam(roots: &[PathBuf]) -> Option<PathBuf> {
    roots
        .iter()
        .flat_map(|root| steam_libraries(root))
        .map(|library| library.join(STEAM_GAME_DIR))
        .find(|dir| dir.join("Bin").join("UI.zip").is_file())
}

/// A Steam root followed by every library its `libraryfolders.vdf` lists.
fn steam_libraries(root: &Path) -> Vec<PathBuf> {
    let vdf = std::fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf")).unwrap_or_default();
    let mut libraries = vec![root.to_path_buf()];
    libraries.extend(library_paths(&vdf));
    libraries
}

/// The `"path"` values in a `libraryfolders.vdf`.
fn library_paths(vdf: &str) -> Vec<PathBuf> {
    vdf.lines()
        .filter_map(|line| {
            let value = line.trim().strip_prefix("\"path\"")?.trim();
            let value = value.strip_prefix('"')?.strip_suffix('"')?;
            Some(PathBuf::from(value.replace("\\\\", "\\")))
        })
        .collect()
}

#[cfg(windows)]
fn steam_roots() -> Vec<PathBuf> {
    use windows::core::w;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ};

    let mut roots = Vec::new();
    let mut buf = [0u16; 1024];
    let mut len = std::mem::size_of_val(&buf) as u32;
    let found = unsafe {
        RegGetValueW(HKEY_CURRENT_USER, w!("Software\\Valve\\Steam"), w!("SteamPath"), RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr().cast()), Some(&mut len))
    };
    if found.is_ok() {
        // `len` is in bytes and counts the terminating null.
        let chars = (len as usize / 2).saturating_sub(1);
        roots.push(PathBuf::from(String::from_utf16_lossy(&buf[..chars])));
    }
    roots.push(PathBuf::from(r"C:\Program Files (x86)\Steam"));
    roots
}

#[cfg(not(windows))]
fn steam_roots() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else { return Vec::new() };
    vec![home.join(".steam/steam"), home.join(".local/share/Steam")]
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

/// Every livery the game has a render for, e.g. `36_26_ALPI20490502`.
pub fn livery_names(game_dir: &Path) -> Vec<String> {
    list_liveries(&game_dir.join("Bin").join("UI.zip")).unwrap_or_default()
}

fn list_liveries(archive_path: &Path) -> Option<Vec<String>> {
    let archive = zip::ZipArchive::new(BufReader::new(File::open(archive_path).ok()?)).ok()?;
    let mut names: Vec<String> = archive
        .file_names()
        .filter_map(|name| {
            let veh = name
                .strip_prefix("start\\images\\cars\\FrontThumbnail\\")?
                .strip_suffix("_frontAngle.webp")?;
            Some(veh.to_owned())
        })
        .collect();
    names.sort();
    Some(names)
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
    fn reads_library_paths_from_steam_config() {
        let vdf = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"\n\t\t\"label\"\t\t\"\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t}\n}\n";
        assert_eq!(library_paths(vdf), [PathBuf::from(r"C:\Program Files (x86)\Steam"), PathBuf::from(r"D:\SteamLibrary")]);
        assert!(library_paths("").is_empty());
    }

    #[test]
    fn finds_the_game_in_a_listed_library() {
        let root = std::env::temp_dir().join(format!("mulsanne-steam-{}", std::process::id()));
        let library = root.join("library");
        let game = library.join(STEAM_GAME_DIR);
        std::fs::create_dir_all(game.join("Bin")).unwrap();
        std::fs::write(game.join("Bin").join("UI.zip"), b"").unwrap();
        std::fs::create_dir_all(root.join("steam").join("steamapps")).unwrap();
        let vdf = format!("\"libraryfolders\"\n{{\n\t\"1\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n}}\n", library.display().to_string().replace('\\', "\\\\"));
        std::fs::write(root.join("steam").join("steamapps").join("libraryfolders.vdf"), vdf).unwrap();

        assert_eq!(find_in_steam(&[root.join("missing"), root.join("steam")]), Some(game));
        assert_eq!(find_in_steam(&[root.join("missing")]), None);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn lists_liveries_that_have_a_thumbnail() {
        let path = std::env::temp_dir().join(format!("mulsanne-liveries-{}.zip", std::process::id()));
        let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
        for view in ["/thumb.36_26_ALPI20490502", "/thumb.12_26_JOTA50003117", "/top.7_26_TOYOT18793560"] {
            zip.start_file(entry_name(view).unwrap().as_str(), zip::write::SimpleFileOptions::default()).unwrap();
        }
        zip.finish().unwrap();

        assert_eq!(list_liveries(&path), Some(vec!["12_26_JOTA50003117".to_owned(), "36_26_ALPI20490502".to_owned()]));
        assert_eq!(list_liveries(Path::new("/nonexistent/UI.zip")), None);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn missing_game_dir_file_means_unknown() {
        assert_eq!(load_game_dir(Path::new("/nonexistent/mulsanne")), None);
    }
}

