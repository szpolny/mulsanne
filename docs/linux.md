# Linux setup and verification

Mulsanne runs natively on x86-64 Linux. The overlay uses X11, including XWayland
inside a Wayland session. XWayland must be installed and enabled; native Wayland
surfaces do not provide the window placement and global shortcuts this overlay
needs. Mulsanne selects the X11 GTK backend by default. Remove an inherited
`GDK_BACKEND=wayland` override if the overlay does not position correctly.
The HUD advertises itself as a dock so tiling compositors leave it floating.
WebKit's fallback buffer renderer is enabled by default for NVIDIA compatibility;
an explicit `WEBKIT_DISABLE_DMABUF_RENDERER` setting takes precedence.

## Run with Le Mans Ultimate

1. Download the Linux AppImage or `.deb` and `lmu-bridge.exe` from the same
   release. Make the AppImage executable with `chmod +x Mulsanne_*.AppImage`.
2. Start LMU through Steam using Proton, in borderless or windowed mode.
3. Run the bridge in that game's prefix using Protontricks:

   ```sh
   protontricks-launch --appid 2399420 "$HOME/Downloads/lmu-bridge.exe"
   ```

   Use the same Proton version configured for LMU. Do not use a separate system
   Wine prefix: it cannot see the game's mappings. Start the game **before** the
   bridge, because Steam can wait for existing prefix processes to exit before
   it starts a game. The bridge exits automatically when LMU closes; run it again
   for the next game session. On an initial mapping delay it waits for LMU.
4. Start Mulsanne. The Live page should report a game connection. Use
   **Ctrl+Shift+F9** to edit the overlay and **Ctrl+Shift+F10** to toggle it. The
   companion's overlay controls remain available if the compositor reserves a
   shortcut. The game's Wine foreground state drives **Hide when unfocused**.

The bridge's default Windows path `Z:\dev\shm\LMU_Data` corresponds to the
native path `/dev/shm/LMU_Data`. A custom path must be accessible to both the
Proton prefix and the native app. For example:

```sh
protontricks-launch --appid 2399420 "$HOME/Downloads/lmu-bridge.exe" 'Z:\tmp\mulsanne-lmu'
MULSANNE_LMU_SHM=/tmp/mulsanne-lmu ./Mulsanne_*.AppImage
```

To include penalty descriptions from the game's trace log, set the **native**
installation directory when starting Mulsanne:

```sh
MULSANNE_LMU_DIR="$HOME/.local/share/Steam/steamapps/common/Le Mans Ultimate" ./Mulsanne_*.AppImage
```

Use your actual Steam library path. Laps are stored in
`${XDG_DATA_HOME:-$HOME/.local/share}/xyz.mulsanne/laps.db` and settings in
`${XDG_CONFIG_HOME:-$HOME/.config}/xyz.mulsanne/config.json`. Debug builds add a
`dev` subdirectory and use separate data.

If the Live page stays disconnected, check the bridge terminal and the mirror's
modification time (`stat /dev/shm/LMU_Data`). The native app rejects incomplete
frames and mirrors older than ten seconds. The app and bridge must come from the
same release. If an AppImage cannot mount because FUSE is unavailable, run it
with `APPIMAGE_EXTRACT_AND_RUN=1`. On NixOS, use `appimage-run` to provide the
standard Linux runtime environment.

## Build and test

On Ubuntu 22.04, install the system dependencies:

```sh
sudo apt-get install build-essential pkg-config libssl-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libdbus-1-dev
pnpm install --frozen-lockfile
pnpm check
pnpm test
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib
pnpm tauri build --ci --no-sign --bundles appimage,deb --config '{"bundle":{"createUpdaterArtifacts":false}}' -- --locked
```

Build with the Tauri CLI so the executable embeds the frontend. A bare
`cargo build` uses the development URL instead. The unsigned build command above
produces local test packages; release CI signs the updater artifacts.

Build and test the bridge on Windows:

```sh
cargo test --locked --manifest-path lmu-bridge/Cargo.toml
cargo build --locked --release --manifest-path lmu-bridge/Cargo.toml
```

Alternatively, cross-compile for `x86_64-pc-windows-gnu` using a Rust toolchain
with that target and a MinGW linker. The Windows lifecycle test can also run
under Wine in a disposable prefix. It creates a simulated game mapping and
window, launches the actual bridge, checks frame updates and cleanup, verifies
normal bridge exit, and attaches a new bridge after a game restart. It does not
launch or change an installed game.

The native smoke test needs `xvfb openbox xcompmgr xdotool imagemagick x11-utils`.
It runs on a private display with temporary app data and a POSIX locale, checks
rendering and X11 input regions, and exercises edit/visibility shortcuts:

```sh
python3 scripts/smoke-linux.py
python3 scripts/smoke-linux.py src-tauri/target/release/bundle/appimage/Mulsanne_0.6.1_amd64.AppImage src-tauri/target/smoke-appimage
```

Use the package filename produced by your build. Screenshots and logs are saved
under the output directory. The Check workflow builds both Linux packages,
runs these smoke tests, and provides the packages and screenshots as artifacts.
It also tests and builds the Windows bridge without requiring release secrets.

## Mirror format

The bridge copies the SDK block while holding the game's cooperative lock. It
writes the `TOTAL_SIZE` raw bytes followed by one byte for foreground state
(`0` or `1`) to a staging file beside the destination, closes it, then atomically
renames it over the destination. Native readers open the destination anew for
each read, so they see complete immutable frames and follow bridge restarts.
No lock or event from wineserver is exposed to the native app.

The pinned uPlot patch in `patches/` catches invalid navigator language tags
before its import-time number formatter can blank the UI. Keep it until the
upstream dependency handles POSIX locales; the frontend regression test covers
both the ESM and CommonJS entry points.

## Verification record

The Linux port was verified with Ubuntu 22.04 AppImage and installed Debian
packages, plus Wine 11.19 Staging for the Windows bridge fixture. Native readers
observed the fixture's complete frames through Wine's Z: drive. The native app
also ran on Hyprland 0.56.2 through XWayland, with a floating, monitor-sized HUD.
Updater signing was checked with a disposable test key, not the release key.

The actual LMU executable was not installed on the verification machine. The
bridge fixture exercises the same named mapping and game-window lifecycle, but
a real driving session remains a separate compatibility check.
