#!/usr/bin/env bash
# Writes the AUR PKGBUILD for a release: render.sh <version> <deb> <license> <out>
set -euo pipefail
version=$1 deb=$2 license=$3 out=$4
sha() { sha256sum -- "$1" | cut -d' ' -f1; }
sed -e "s/@VERSION@/$version/" \
    -e "s/@DEB_SHA256@/$(sha "$deb")/" \
    -e "s/@LICENSE_SHA256@/$(sha "$license")/" \
    "$(dirname -- "$0")/PKGBUILD.in" > "$out"
