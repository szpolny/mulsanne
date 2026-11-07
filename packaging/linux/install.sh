#!/usr/bin/env bash
# Per-user installation; no sudo, downloads, or changes to the game.
set -euo pipefail
source_dir=$(cd -- "$(dirname -- "$0")" && pwd)
base=${XDG_DATA_HOME:-$HOME/.local/share}/mulsanne-linux
apps=${XDG_DATA_HOME:-$HOME/.local/share}/applications
native=''
if [[ ${1:-} == --uninstall && $# == 1 ]]; then
  rm -f -- "$apps/xyz.mulsanne.linux.desktop"
  rm -rf -- "$base"
  echo 'Removed Linux launcher. Settings and laps were kept.'
  exit 0
fi
if [[ ${1:-} == --native && $# == 2 ]]; then
  native=$(realpath -- "$2")
  [[ -x $native ]] || { echo 'Native Mulsanne executable not found.' >&2; exit 1; }
elif (($#)); then
  echo 'Usage: install.sh [--native /usr/bin/lm-overlay | --uninstall]' >&2
  exit 2
fi
for tool in bash flock setsid realpath; do
  command -v "$tool" >/dev/null || { echo "Missing $tool; install bash, coreutils and util-linux." >&2; exit 1; }
done
for file in launch.sh run-lmu-bridge.sh hyprland.lua lmu-bridge.exe icon.png; do
  [[ -f $source_dir/$file ]] || { echo "Missing release file: $file" >&2; exit 1; }
done
[[ -n $native || -f $source_dir/Mulsanne.AppImage ]] || { echo 'Missing Mulsanne.AppImage.' >&2; exit 1; }
mkdir -p -- "$base" "$apps"
if [[ $source_dir != "$base" ]]; then
  for file in launch.sh run-lmu-bridge.sh hyprland.lua lmu-bridge.exe icon.png install.sh; do
    install -m 644 -- "$source_dir/$file" "$base/$file"
  done
  if [[ -z $native ]]; then install -m 755 -- "$source_dir/Mulsanne.AppImage" "$base/Mulsanne.AppImage"; fi
fi
chmod +x -- "$base/launch.sh" "$base/run-lmu-bridge.sh" "$base/install.sh"
if [[ -n $native ]]; then printf '%s\n' "$native" > "$base/native-binary"; else rm -f -- "$base/native-binary"; fi
# Escape desktop-entry quoted paths, including spaces and shell metacharacters.
escaped=${base//\\/\\\\}
escaped=${escaped//\"/\\\"}
escaped=${escaped//\$/\\\$}
escaped=${escaped//\`/\\\`}
escaped=${escaped//%/%%}
cat > "$apps/xyz.mulsanne.linux.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Mulsanne (Experimental Linux)
Comment=LMU telemetry companion and fullscreen HUD
Exec=bash "$escaped/launch.sh"
Icon=$base/icon.png
Terminal=false
Categories=Game;
StartupNotify=false
DESKTOP
command -v update-desktop-database >/dev/null && update-desktop-database "$apps" || true
printf 'Installed Mulsanne (Experimental Linux). Start LMU through Steam, then open Mulsanne from your app menu.\nLogs: %s\n' "${XDG_STATE_HOME:-$HOME/.local/state}/mulsanne"
