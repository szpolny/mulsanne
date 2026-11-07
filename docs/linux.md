# Experimental Linux support

Mulsanne runs natively on x86-64 Linux while LMU runs through Steam/Proton.
Fullscreen has been tested on NixOS with Hyprland/XWayland. Other desktops are
experimental; native Wayland and Flatpak Steam have not been verified.

## NixOS / Home Manager

Add Mulsanne to your flake (use the branch below until the PR is merged):

```nix
inputs.mulsanne = {
  url = "github:szpolny/mulsanne/feat/linux-support";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Then add the package to `environment.systemPackages` or `home.packages`:

```nix
inputs.mulsanne.packages.${pkgs.stdenv.hostPlatform.system}.default
```

Updates follow your normal flake input updates. No AppImage or separate bridge
installation is needed. To try it from the checkout: `nix run .`.

## Debian / Ubuntu and Fedora / openSUSE

Install the matching release package:

```sh
sudo apt install ./Mulsanne_VERSION_amd64.deb
# Fedora:
sudo dnf install ./Mulsanne-VERSION-1.x86_64.rpm
# openSUSE:
sudo zypper install ./Mulsanne-VERSION-1.x86_64.rpm
```

The packages include the app, launcher and bridge. They can be upgraded/removed
with the package manager. Automatic apt/dnf update discovery requires a package
repository; one is not provided yet.

## Run

Start **Mulsanne** from the app menu and LMU through Steam, in either order.
The launcher attaches the bridge automatically. On Hyprland it also configures
fullscreen stacking, alignment and transparency without changing config files.
XWayland must be enabled. The HUD stays visible while using another monitor and
hides when switching away from the game on its monitor.

- **Ctrl+Shift+F9**: edit the layout.
- **Ctrl+Shift+F10**: hide/show the HUD.
- Widgets are hidden off track by default; enter a session or open the editor.

Logs are in `${XDG_STATE_HOME:-$HOME/.local/state}/mulsanne/`. Settings and laps
remain in the standard XDG config/data directories under `xyz.mulsanne` when the
package is removed. To include penalty descriptions from the trace log, set
`MULSANNE_LMU_DIR` to the native LMU installation directory.
