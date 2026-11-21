use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

use serde::{de::DeserializeOwned, Deserialize};

use super::Bounds;

pub(super) struct Compositor(PathBuf);

#[derive(Deserialize)]
struct Client {
    address: String,
    class: String,
    title: String,
    mapped: bool,
    hidden: bool,
    at: [i32; 2],
    size: [u32; 2],
}

impl Client {
    fn game(&self) -> bool {
        self.class.eq_ignore_ascii_case("steam_app_2399420")
            || (self.class.eq_ignore_ascii_case("gamescope")
                && (self.title == "Le Mans Ultimate"
                    || self.title.starts_with("Le Mans Ultimate v")))
    }
}

#[derive(Deserialize)]
struct ActiveWindow {
    // Hyprland returns an empty object when no window has focus.
    address: Option<String>,
}

impl Compositor {
    pub(super) fn new() -> Option<Self> {
        let path = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?)
            .join("hypr")
            .join(std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?)
            .join(".socket.sock");
        Some(Self(path))
    }

    fn query<T: DeserializeOwned>(&self, request: &[u8]) -> Option<T> {
        let mut stream = UnixStream::connect(&self.0).ok()?;
        let timeout = Some(Duration::from_millis(100));
        stream.set_read_timeout(timeout).ok()?;
        stream.set_write_timeout(timeout).ok()?;
        stream.write_all(request).ok()?;
        let mut reply = Vec::new();
        stream.read_to_end(&mut reply).ok()?;
        serde_json::from_slice(&reply).ok()
    }

    pub(super) fn game_window(&self) -> Option<(Option<Bounds>, bool)> {
        let clients: Vec<Client> = self.query(b"j/clients")?;
        let mut games = clients.iter().filter(|client| client.game()).peekable();
        games.peek()?;
        let active = self
            .query::<ActiveWindow>(b"j/activewindow")
            .and_then(|window| window.address);
        Some(game_view(games, active.as_deref()))
    }
}

fn game_view<'a>(
    games: impl Iterator<Item = &'a Client>,
    active: Option<&str>,
) -> (Option<Bounds>, bool) {
    let window = games
        .filter(|client| client.mapped && !client.hidden && !client.size.contains(&0))
        .max_by_key(|client| {
            let [width, height] = client.size;
            (
                active == Some(client.address.as_str()),
                u64::from(width) * u64::from(height),
            )
        });
    let focused = window.is_some_and(|client| active == Some(client.address.as_str()));
    let bounds = window.map(|client| {
        let [x, y] = client.at;
        let [width, height] = client.size;
        Bounds {
            x,
            y,
            width,
            height,
        }
    });
    (bounds, focused)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_the_outer_gamescope_window_and_its_focus() {
        let clients: Vec<Client> = serde_json::from_str(
            r#"[
            {"address":"0x1","class":"gamescope","title":"Le Mans Ultimate v1.4200",
             "mapped":true,"hidden":false,"at":[1920,0],"size":[2560,1440]},
            {"address":"0x2","class":"gamescope","title":"Another game",
             "mapped":true,"hidden":false,"at":[0,0],"size":[1920,1080]}
        ]"#,
        )
        .unwrap();
        let games = || clients.iter().filter(|client| client.game());
        let bounds = Some(Bounds {
            x: 1920,
            y: 0,
            width: 2560,
            height: 1440,
        });
        assert_eq!(game_view(games(), Some("0x1")), (bounds, true));
        assert_eq!(game_view(games(), Some("0x2")), (bounds, false));
        assert_eq!(game_view(games(), None), (bounds, false));
    }
}
