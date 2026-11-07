# Experimental Linux support

Linux support is **experimental**, for x86-64 PCs. LMU runs through Steam/Proton;
Mulsanne runs natively using X11 or XWayland. The installer starts the telemetry
bridge automatically, so there is no separate bridge terminal to manage.

Fullscreen, click-through, and multiple monitors have been tested on **NixOS
with Hyprland 0.57.1 and XWayland**. The Hyprland integration keeps the HUD aligned
with fullscreen LMU, removes compositor blur and decorations, and keeps it visible
when using another monitor. Switching workspaces on the game's monitor or covering
the game with another app hides the HUD. It does not take keyboard focus merely
because the pointer enters it. The companion's layout editor and visibility
controls remain available alongside Ctrl+Shift+F9 and Ctrl+Shift+F10.

Other desktops are experimental and have not received the same fullscreen
verification. Native Wayland surfaces are not supported. XWayland must be enabled;
fullscreen behavior on GNOME/KDE and exclusive fullscreen require further testing.
Steam installed through the distribution is the tested configuration; Flatpak
Steam's sandbox and permissions have not been verified.

## Install from a release

Download `Mulsanne_VERSION_linux-experimental_x86_64.tar.gz` and its `.sha256`
file from the [same release](https://github.com/szpolny/mulsanne/releases).
These archives become available after this change is merged and a release is
published; older Windows-only releases do not contain them.

In the download directory, replacing `VERSION` with the version you downloaded:

```sh
sha256sum -c Mulsanne_VERSION_linux-experimental_x86_64.tar.gz.sha256
tar -xzf Mulsanne_VERSION_linux-experimental_x86_64.tar.gz
cd mulsanne-linux-experimental
bash install.sh
```

Open **Mulsanne (Experimental Linux)** from the app menu. Start LMU through Steam
before or after opening Mulsanne; the bridge attaches once the game starts and
reattaches after a game restart. Fullscreen works with the supplied Hyprland
integration. Enter a session and drive to see widgets: **Hide overlay off track**
is enabled by default. Ctrl+Shift+F9 opens the layout editor outside a session.

The installer needs Bash, coreutils, and util-linux (`flock`, `setsid`, `nsenter`).
It installs to `${XDG_DATA_HOME:-$HOME/.local/share}/mulsanne-linux` without sudo.
The archive includes the matching app and bridge. Keep them from the same release.

### Ubuntu, Debian, Linux Mint, and Pop!_OS

Use the archive installer above. On Ubuntu 24.04 and newer, AppImages may need:

```sh
sudo apt install libfuse2t64
```

On Ubuntu 22.04 and Debian releases providing it under the older name:

```sh
sudo apt install libfuse2
```

Alternatively, install the release's `.deb` (Ubuntu 22.04+/Debian 12+), then install
the launcher from the matching extracted archive:

```sh
sudo apt install ./Mulsanne_VERSION_amd64.deb
bash install.sh --native /usr/bin/lm-overlay
```

Select **Mulsanne (Experimental Linux)** in the menu to use automatic bridge
attachment and compositor integration. The package's plain Mulsanne entry runs
only the companion and HUD.

### Fedora, Arch/Manjaro, and openSUSE

Use the same archive installer; there is no published RPM, AUR, or OBS package.
If AppImage mounting is unavailable, extraction mode avoids the FUSE dependency:

```sh
APPIMAGE_EXTRACT_AND_RUN=1 bash "$HOME/.local/share/mulsanne-linux/launch.sh"
```

That variable must also be present when launching the app. To persist it for
Mulsanne, create this file before starting the app:

```sh
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/mulsanne"
printf '%s\n' 'APPIMAGE_EXTRACT_AND_RUN=1' > "${XDG_CONFIG_HOME:-$HOME/.config}/mulsanne/launcher.env"
```

The launcher reads this setting as data, not executable shell code. The same
archive approach applies to immutable Fedora desktops and SteamOS desktop mode;
these have not been tested individually. You may need to install or enable
XWayland using your distribution's package manager.

### NixOS and Nix

On NixOS the launcher runs the bundled AppImage through `appimage-run`:

```sh
nix profile install nixpkgs#appimage-run
bash install.sh
```

For a declarative NixOS or Home Manager configuration, add `pkgs.appimage-run`
to `environment.systemPackages` or `home.packages`. Enable XWayland in your
compositor configuration (for Hyprland: `programs.hyprland.xwayland.enable = true`).
Apply configuration changes using your normal system-management workflow.
The installer does not switch or modify your NixOS configuration.

For a one-off run without installing the runtime into your profile:

```sh
nix shell nixpkgs#appimage-run --command bash "$HOME/.local/share/mulsanne-linux/launch.sh"
```

Nix on other Linux distributions can use the normal archive installer. This is
a per-user AppImage installation, not a package published in nixpkgs.

[NixOS AppImage support](https://nixos.org/manual/nixos/stable/#sec-appimage) and
[AppImage FUSE troubleshooting](https://docs.appimage.org/user-guide/troubleshooting/fuse.html)
explain the runtime options in more detail.

## Updates and uninstall

Close Mulsanne, extract the new archive, and run `bash install.sh` again. For a
native `.deb` installation, update the `.deb` and use `--native` again. Linux
updates are manual during the experimental period so the app, bridge and
compositor integration stay together. Existing settings and laps are preserved.

```sh
bash "$HOME/.local/share/mulsanne-linux/install.sh" --uninstall
```

This removes the launcher and app-menu entry, retaining settings and lap data.
Hyprland rules are applied to the running session by the launcher; no compositor
configuration files are modified. Reloading Hyprland clears these temporary rules.

## Troubleshooting and data

Logs: `${XDG_STATE_HOME:-$HOME/.local/state}/mulsanne/app.log` and `bridge.log`.
If disconnected, inspect the bridge log and `stat /dev/shm/LMU_Data`. The native
app rejects incomplete frames and mirrors older than ten seconds. Do not start
a separate system Wine prefix: it cannot see the game's mappings. The bridge
uses the running game's Wine loader, prefix and namespaces, including Steam's
NixOS FHS environment. It runs as your normal user and never needs sudo.

To launch the bridge manually:

```sh
bash run-lmu-bridge.sh /path/to/lmu-bridge.exe
```

An optional second argument sets the Windows mirror path; set the corresponding
native `MULSANNE_LMU_SHM` path for Mulsanne. For example, `Z:\dev\shm\custom-lmu`
corresponds to `/dev/shm/custom-lmu`.

Laps: `${XDG_DATA_HOME:-$HOME/.local/share}/xyz.mulsanne/laps.db`.
Settings: `${XDG_CONFIG_HOME:-$HOME/.config}/xyz.mulsanne/config.json`.
Debug builds use a separate `dev` subdirectory. To include penalty descriptions
from the game's trace log, export `MULSANNE_LMU_DIR` with the native LMU install
path before starting Mulsanne. The normal Steam library path is
`$HOME/.local/share/Steam/steamapps/common/Le Mans Ultimate`.

The app selects the X11 GTK backend. WebKit uses shared-memory buffer transport
for NVIDIA compatibility unless explicitly overridden. Avoid
`WEBKIT_DISABLE_DMABUF_RENDERER=1` on WebKitGTK 2.54: its transparent fallback can
lose unchanged pixels. A transparent repaint canvas is enabled only when GTK
cannot create a GL context or compositing is disabled. It pauses while widgets
are hidden and uses more CPU than composited rendering.

## Build and test

On Ubuntu 22.04:

```sh
sudo apt install build-essential pkg-config libssl-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libdbus-1-dev libx11-dev libxext-dev
pnpm install --frozen-lockfile
pnpm check
pnpm test
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib
pnpm tauri build --ci --no-sign --bundles appimage,deb --config '{"bundle":{"createUpdaterArtifacts":false}}' -- --locked
```

Build with the Tauri CLI so the executable embeds the frontend. A bare Cargo
build uses the development URL. Build the bridge on Windows:

```sh
cargo test --locked --manifest-path lmu-bridge/Cargo.toml
cargo build --locked --release --manifest-path lmu-bridge/Cargo.toml
```

Alternatively use Rust's `x86_64-pc-windows-gnu` target with a MinGW linker.
The bridge lifecycle fixture can run under Wine in a disposable prefix without
launching or changing an installed game.

Native smoke tests need `xvfb openbox xcompmgr xdotool imagemagick x11-utils`,
Node.js, a C compiler and X11/Xext development libraries:

```sh
node scripts/smoke-linux.mjs
node scripts/smoke-linux.mjs /path/to/Mulsanne.AppImage src-tauri/target/smoke-appimage
bash scripts/package-linux.sh /path/to/Mulsanne.AppImage /path/to/lmu-bridge.exe /tmp/mulsanne-linux-experimental.tar.gz
```

Smoke tests use a private display, temporary data and a POSIX locale. They
exercise changing telemetry, static label rendering, game moves/resizes, actual
X11 input regions, editing, shortcuts, hide/show and lap database creation.
The Check workflow tests both Linux packages and uploads packages and screenshots;
the bridge job tests and builds the Windows executable without release secrets.
No Python runtime is required for installation, launching or tests.

## Verification record

Ubuntu 22.04 AppImage and installed Debian packages were verified with Wine
11.19 Staging for the bridge fixture. Real-game testing on 2026-11-01 used LMU
v1.4200 (Steam build 25661166), GE-Proton10-34-LMU-hid_fixes and NixOS Hyprland/
XWayland: player telemetry reached the native parser, companion and HUD. Session
time, RPM, fuel, tyre temperatures and focus state updated. A subsequent
WebKitGTK 2.54.1 check verified the rendering fallback and game-client bounds.

On 2026-11-07 the user confirmed correct fullscreen behavior at 2560×1440 on
Hyprland 0.57.1, including alignment, no blur/decorations, and continued display
while using a second monitor. Switching to another workspace on the game's
monitor hides the HUD. This confirms the supplied experimental Hyprland setup;
it does not establish support for every Linux desktop or distribution.
No completed real-game lap was recorded in these checks; recorder tests cover
lap storage. Release-key signing and public distribution remain release checks.

## Mirror format

The bridge copies the SDK block under the game's cooperative lock, writes the
324824 raw bytes plus one foreground-state byte to a staging file, closes it,
and atomically renames it over the destination. Native readers reopen each frame,
so they see complete immutable snapshots and follow bridge restarts. No
wineserver lock/event is exposed to the native app.
