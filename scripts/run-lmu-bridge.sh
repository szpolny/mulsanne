#!/usr/bin/env bash
# Attach only to the running game's Proton runtime, never a separate Wine prefix.
set -euo pipefail

find_game() {
  local proc_root=${1:-/proc} process command_line executable entry kind
  GAME_PID='' GAME_WINE='' GAME_PREFIX=''
  GAME_RUNTIME=() GAME_NAMESPACES=()
  for process in "$proc_root"/[0-9]*; do
    [[ -O $process && -r $process/cmdline && -r $process/environ ]] || continue
    command_line=''
    IFS= read -r -d '' command_line < "$process/cmdline" || continue
    executable=${command_line//\\//}
    [[ ${executable##*/} == 'Le Mans Ultimate.exe' ]] || continue
    GAME_RUNTIME=() GAME_WINE='' GAME_PREFIX=''
    while IFS= read -r -d '' entry; do
      case $entry in
        WINELOADER=*) GAME_WINE=${entry#*=}; GAME_RUNTIME+=("$entry");;
        WINEPREFIX=*) GAME_PREFIX=${entry#*=}; GAME_RUNTIME+=("$entry");;
        WINEFSYNC=*|WINEESYNC=*|WINEDLLPATH=*|LD_LIBRARY_PATH=*|DISPLAY=*|XAUTHORITY=*|XDG_RUNTIME_DIR=*|PATH=*) GAME_RUNTIME+=("$entry");;
      esac
    done < "$process/environ"
    [[ -n $GAME_WINE && -n $GAME_PREFIX ]] || continue
    GAME_PID=${process##*/}
    for kind in user mnt; do
      if [[ $(readlink "$process/ns/$kind") != "$(readlink "$proc_root/self/ns/$kind")" ]]; then
        case $kind in user) GAME_NAMESPACES+=(--user);; mnt) GAME_NAMESPACES+=(--mount);; esac
      fi
    done
    return 0
  done
  return 1
}

run_bridge() {
  local bridge=$1
  shift
  bridge=$(realpath -- "$bridge")
  [[ -f $bridge ]] || { echo "Bridge executable not found: $bridge" >&2; return 1; }
  find_game || { echo 'Start Le Mans Ultimate through Steam before starting the bridge.' >&2; return 1; }
  printf 'Attaching bridge to LMU process %s\n' "$GAME_PID"
  # Wine must inherit pipes: host log files can crash Proton in an isolated runtime.
  if ((${#GAME_NAMESPACES[@]})); then
    command -v nsenter >/dev/null || { echo 'Install util-linux (nsenter) to attach to the Steam runtime.' >&2; return 1; }
    nsenter --target "$GAME_PID" "${GAME_NAMESPACES[@]}" --preserve-credentials --root --wd -- \
      /usr/bin/env "${GAME_RUNTIME[@]}" "WINEDEBUG=${WINEDEBUG:--all}" "$GAME_WINE" "$bridge" "$@" </dev/null 2>&1 | cat
  else
    env "${GAME_RUNTIME[@]}" LD_PRELOAD= "WINEDEBUG=${WINEDEBUG:--all}" "$GAME_WINE" "$bridge" "$@" </dev/null 2>&1 | cat
  fi
}

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  if (($# < 1 || $# > 2)); then
    echo 'Usage: run-lmu-bridge.sh /path/to/lmu-bridge.exe [Windows-mirror-path]' >&2
    exit 2
  fi
  run_bridge "$@"
fi
