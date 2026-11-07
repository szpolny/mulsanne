#!/usr/bin/env bash
# Starts the native companion/HUD and attaches the bridge when LMU is running.
set -euo pipefail
base=$(cd -- "$(dirname -- "$0")" && pwd)
state=${XDG_STATE_HOME:-$HOME/.local/state}/mulsanne
mkdir -p -- "$state"
exec 9>"$state/launcher.lock"
flock -n 9 || exit 0
if [[ -n ${HYPRLAND_INSTANCE_SIGNATURE:-} ]] && command -v hyprctl >/dev/null; then
  hyprctl eval "$(cat "$base/hyprland.lua")" >>"$state/app.log" 2>&1 || true
fi
# Read documented settings as data, never as executable shell code.
settings=${XDG_CONFIG_HOME:-$HOME/.config}/mulsanne/launcher.env
if [[ -f $settings ]]; then
  while IFS= read -r setting; do
    case $setting in APPIMAGE_EXTRACT_AND_RUN=1) export APPIMAGE_EXTRACT_AND_RUN=1;; esac
  done < "$settings"
fi
app_pid='' bridge_pid=''
cleanup() {
  trap - EXIT INT TERM
  if [[ -n ${HYPRLAND_INSTANCE_SIGNATURE:-} ]] && command -v hyprctl >/dev/null; then
    hyprctl eval 'if _G.mulsanne_linux_timer then _G.mulsanne_linux_timer:set_enabled(false) end; if _G.mulsanne_linux_rule then _G.mulsanne_linux_rule:set_enabled(false) end' >>"$state/app.log" 2>&1 || true
  fi
  for child in "$bridge_pid" "$app_pid"; do
    [[ -n $child ]] || continue
    kill -TERM -- "-$child" 2>/dev/null || true
  done
}
trap cleanup EXIT
trap 'exit 0' INT TERM
app=("$base/Mulsanne.AppImage")
if [[ -f $base/native-binary ]]; then
  IFS= read -r native < "$base/native-binary"
  app=("$native")
elif [[ -e /etc/NIXOS ]]; then
  if command -v appimage-run >/dev/null; then
    app=(appimage-run "${app[@]}")
  else
    echo 'Install appimage-run first: nix profile install nixpkgs#appimage-run' >&2
    command -v notify-send >/dev/null && notify-send Mulsanne 'Install appimage-run, then relaunch Mulsanne.' || true
    exit 1
  fi
fi
setsid env GDK_BACKEND=x11 "${app[@]}" >>"$state/app.log" 2>&1 9>&- &
app_pid=$!
mirror=${MULSANNE_LMU_SHM:-/dev/shm/LMU_Data}
windows_mirror="Z:${mirror//\//\\}"
while kill -0 "$app_pid" 2>/dev/null; do
  if [[ -z $bridge_pid ]] || ! kill -0 "$bridge_pid" 2>/dev/null; then
    if [[ -n $bridge_pid ]]; then wait "$bridge_pid" || true; fi
    setsid bash "$base/run-lmu-bridge.sh" "$base/lmu-bridge.exe" "$windows_mirror" >>"$state/bridge.log" 2>&1 9>&- &
    bridge_pid=$!
  fi
  sleep 5
done
wait "$app_pid" || true
