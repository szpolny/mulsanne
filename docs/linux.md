# Experimental Linux support

Mulsanne runs natively on x86-64 Linux while LMU runs through Steam/Proton.

The HUD is drawn above the fullscreen game on Wayland desktops that support the
layer shell protocol, such as Hyprland, Sway and other wlroots compositors, KDE
Plasma, niri and COSMIC. It has been tested on NixOS with Hyprland. GNOME and
X11 sessions aren't supported yet: the app starts there, but the HUD may not
stay above the game. Flatpak Steam has not been verified.

## NixOS / Home Manager

Add Mulsanne to your flake:

```nix
inputs.mulsanne = {
  url = "github:szpolny/mulsanne";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Then add the package to `environment.systemPackages` or `home.packages`:

```nix
inputs.mulsanne.packages.${pkgs.stdenv.hostPlatform.system}.default
```

Updates follow your normal flake input updates. No AppImage or separate bridge
installation is needed. To try it from the checkout: `nix run .`.

## Debian / Ubuntu

Add the Mulsanne repository once, then install:

```sh
sudo curl -fsSLo /usr/share/keyrings/mulsanne.gpg https://repo.mulsanne.xyz/mulsanne.gpg
echo 'deb [arch=amd64 signed-by=/usr/share/keyrings/mulsanne.gpg] https://repo.mulsanne.xyz/deb stable main' \
  | sudo tee /etc/apt/sources.list.d/mulsanne.list
sudo apt update && sudo apt install mulsanne
```

New releases arrive with `sudo apt update && sudo apt upgrade`.

## Fedora

```sh
sudo curl -fsSLo /etc/yum.repos.d/mulsanne.repo https://repo.mulsanne.xyz/mulsanne.repo
sudo dnf install mulsanne
```

New releases arrive with `sudo dnf upgrade`.

## openSUSE

```sh
sudo zypper addrepo https://repo.mulsanne.xyz/mulsanne.repo
sudo zypper refresh && sudo zypper install mulsanne
```

New releases arrive with `sudo zypper update`.

## Arch Linux

There is no AUR package yet, because AUR account registration is currently
closed. A `mulsanne-bin` package will be added once registration reopens. Until
then, use the Nix flake above, or build the package yourself from the AUR
recipe in this repository (in bash or zsh):

```sh
git clone --depth 1 https://github.com/szpolny/mulsanne
cd mulsanne/packaging/aur
version=$(curl -fsSL https://api.github.com/repos/szpolny/mulsanne/releases/latest \
  | sed -n 's/.*"tag_name": *"v\([^"]*\)".*/\1/p')
sed -e "s/@VERSION@/$version/" -e 's/@[A-Z_]*SHA256@/SKIP/' PKGBUILD.in > PKGBUILD
makepkg -si
```

This installs the latest release as `mulsanne-bin`, which pacman can remove like
any other package. It doesn't update on its own: repeat these steps in a fresh
clone to install a new release.

## Manual install

Each [release](https://github.com/szpolny/mulsanne/releases/latest) also has the
`.deb` and `.rpm` files, which you can install directly (for example,
`sudo apt install ./Mulsanne_VERSION_amd64.deb`). These won't update
automatically.

The packages include the app, launcher and bridge, and can be removed with the
package manager. Mulsanne does not update itself on Linux. It checks GitHub for
a newer release on launch and shows an "Update available" notice in the sidebar
and Settings, so you know to update through your package manager.

## Run

Start **Mulsanne** from the app menu and LMU through Steam, in either order.
The launcher attaches the bridge automatically. No compositor configuration is
needed. XWayland must be enabled, because the game runs through it. The HUD
covers the game's window and hides while the game isn't focused.

- **Ctrl+Shift+F9**: edit the layout.
- **Ctrl+Shift+F10**: hide/show the HUD.

Both shortcuts can be changed in Settings. They work while the game or a
Mulsanne window is focused, not in other apps.
- Widgets are hidden off track by default; enter a session or open the editor.

The companion uses a GTK title bar by default and follows your GTK button theme
and layout. If a compositor plugin such as hyprbars adds its own bar, choose
**None** under **Settings → General → Title bar**, or exclude the `lm-overlay` window from
the plugin to keep the GTK bar. Title bar changes take effect on restart.

Logs are in `${XDG_STATE_HOME:-$HOME/.local/state}/mulsanne/`. Settings and laps
remain in the standard XDG config/data directories under `xyz.mulsanne` when the
package is removed. To include penalty descriptions from the trace log, set
`MULSANNE_LMU_DIR` to the native LMU installation directory.
