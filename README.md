![Mulsanne logo](src-tauri/icons/128x128.png)

# Mulsanne

Mulsanne is a customizable in-game overlay and telemetry companion for **Le Mans Ultimate**. It reads the game's built-in shared memory, so no game plugin is needed.

Website: [mulsanne.xyz](https://mulsanne.xyz)

## Features

- A movable, scalable HUD with live timing, standings, relative gaps, fuel and energy estimates, tyres, flags, and a track map.
- Automatic lap recording with personal bests, telemetry traces, and lap comparison in the companion app.
- An upcoming daily race schedule from RaceControl, shown in your local time without running the game.
- Demo data for arranging the overlay without running the game.

## Use

1. Run Le Mans Ultimate in **borderless** or windowed mode.
2. Start Mulsanne. The companion app opens, and the overlay appears on your primary monitor.
3. Press **Ctrl+Shift+F9** to edit the layout. Drag widgets to move them and scroll to resize them; press it again to finish.
4. Press **Ctrl+Shift+F10** to show or hide the overlay. Configure widgets on the **Overlays** page.

Use **Demo data** on the **Live** page to arrange the overlay without running the game. Laps are saved locally in `%APPDATA%\xyz.mulsanne\laps.db`; valid laps count toward your personal bests.

## Linux

Mulsanne builds natively on Linux (AppImage and `.deb`) and uses X11 or XWayland for the overlay. LMU runs under Proton; run the release's `lmu-bridge.exe` inside the game's prefix to share telemetry and focus state with the native app. See [Linux setup and verification](docs/linux.md) for launch commands, dependencies, and tests.

## Feedback

Use **Report issue** at the bottom of the companion sidebar or [open an issue on GitHub](https://github.com/szpolny/mulsanne/issues/new/choose) to report a bug or request a feature. For bugs, include your Mulsanne version (shown in Settings), operating system and display session (X11/Wayland on Linux), and steps to reproduce the problem.

## Development

Built with Tauri 2, Rust, Svelte 5, and SQLite.

```bash
pnpm install
pnpm tauri dev
```

Run checks with `pnpm check` and `cargo test --manifest-path src-tauri/Cargo.toml`.

## Releases

Download the installer from [GitHub Releases](https://github.com/szpolny/lm-overlay/releases). The app checks for updates on launch and offers to install them.

Releases are automated with [release-please](https://github.com/googleapis/release-please). Use [conventional commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `feat!:`) on `master`. release-please keeps a release PR open with the next version and changelog, and merging that PR tags the release and builds the signed installer. The version lives in `package.json` only.

## License

[MIT](LICENSE)
