![Mulsanne logo](src-tauri/icons/128x128.png)

# Mulsanne

<p>
  <a href="https://github.com/szpolny/mulsanne/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/szpolny/mulsanne?style=for-the-badge&labelColor=131317&color=e3101f&label=Latest&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjQgMTAgNTggNDQiPjxwYXRoIGQ9Ik02IDUyIDIyIDEyaDEwTDE2IDUyeiIgZmlsbD0iI2ZmZiIvPjxwYXRoIGQ9Ik0yMCA1MiAzNiAxMmgxMEwzMCA1MnoiIGZpbGw9IiNlMzEwMWYiLz48cGF0aCBkPSJNMzQgNTIgNTAgMTJoMTBMNDQgNTJ6IiBmaWxsPSIjZTMxMDFmIiBvcGFjaXR5PSIuNTUiLz48L3N2Zz4%3D"></a>
  <a href="https://github.com/szpolny/mulsanne/releases/latest"><img alt="Windows x64 installer" src="https://img.shields.io/badge/Windows-x64%20installer-24242b?style=for-the-badge&labelColor=131317&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI%2BPHBhdGggZD0iTTAgMGgxMS40djExLjRIMHpNMTIuNiAwSDI0djExLjRIMTIuNnpNMCAxMi42aDExLjRWMjRIMHpNMTIuNiAxMi42SDI0VjI0SDEyLjZ6IiBmaWxsPSIjZjRmNGY2Ii8%2BPC9zdmc%2B"></a>
  <a href="docs/linux.md"><img alt="Linux: deb, rpm and Nix packages (experimental)" src="https://img.shields.io/badge/Linux-deb%20%C2%B7%20rpm%20%C2%B7%20nix-24242b?style=for-the-badge&labelColor=131317&logo=linux&logoColor=f4f4f6"></a>
</p>

Mulsanne is a customizable in-game overlay and telemetry companion for **Le Mans Ultimate**. It reads the game's built-in shared memory, so no game plugin is needed.

Website: [mulsanne.xyz](https://mulsanne.xyz)

## Features

- A movable, scalable HUD with live timing, standings, relative gaps, fuel and energy estimates, tyres, flags, and a track map.
- Automatic lap recording with personal bests, telemetry traces, and lap comparison in the companion app.
- An upcoming daily race schedule from RaceControl, shown in your local time without running the game.
- Demo data for arranging the overlay without running the game.

## Screenshots

<img alt="Mulsanne overlay with standings, relative gaps, fuel, tyres and a track map" src="docs/screenshots/overlay.png">

<details>
<summary><strong>More screenshots</strong></summary>
<br>

<img alt="Comparing two laps on the Analysis page" src="docs/screenshots/analysis.png">

<img alt="Live page" src="docs/screenshots/live.png" width="49%"> <img alt="Overlays page" src="docs/screenshots/overlays.png" width="49%">
<img alt="Sessions page" src="docs/screenshots/sessions.png" width="49%"> <img alt="Daily race schedule" src="docs/screenshots/schedule.png" width="49%">

</details>

Screenshots use the built-in demo data.

## Use

1. Run Le Mans Ultimate. On Windows, use **borderless** or windowed mode; see the Linux section below for experimental fullscreen support.
2. Start Mulsanne. The companion app opens, and the overlay appears on your primary monitor.
3. Press **Ctrl+Shift+F9** to edit the layout. Drag widgets to move them and scroll to resize one (Shift + scroll resizes all of them); press it again to finish.
4. Press **Ctrl+Shift+F10** to show or hide the overlay. Configure widgets on the **Overlays** page.

Use **Demo data** on the **Live** page to arrange the overlay without running the game. Laps are saved locally in `%APPDATA%\xyz.mulsanne\laps.db`; valid laps count toward your personal bests.

## Experimental Linux support

Experimental native Linux packages include the Proton bridge and fullscreen HUD integration tested on Hyprland/XWayland. Install with the Nix flake, `.deb`, or `.rpm`; see [Linux installation](docs/linux.md).

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
