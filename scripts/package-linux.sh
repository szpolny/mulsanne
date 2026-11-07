#!/usr/bin/env bash
set -euo pipefail
if (($# != 3)); then
  echo 'Usage: package-linux.sh app.AppImage lmu-bridge.exe output.tar.gz' >&2
  exit 2
fi
root=$(cd -- "$(dirname -- "$0")/.." && pwd)
output=$(realpath -m -- "$3")
staging=$(mktemp -d)
trap 'rm -rf -- "$staging"' EXIT
base=$staging/mulsanne-linux-experimental
mkdir -p -- "$base" "$(dirname -- "$output")"
cp -- "$1" "$base/Mulsanne.AppImage"
cp -- "$2" "$base/lmu-bridge.exe"
cp -- "$root"/packaging/linux/* "$base/"
cp -- "$root/scripts/run-lmu-bridge.sh" "$base/"
cp -- "$root/src-tauri/icons/128x128.png" "$base/icon.png"
cp -- "$root/docs/linux.md" "$base/README.md"
chmod +x -- "$base"/*.sh "$base/Mulsanne.AppImage"
tar -czf "$output" -C "$staging" mulsanne-linux-experimental
printf 'Created %s\n' "$output"
