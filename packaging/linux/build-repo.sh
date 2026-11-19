#!/usr/bin/env bash
# Builds the signed apt and dnf/zypper repositories served from GitHub Pages.
#
# Usage: build-repo.sh <packages-dir> <output-dir> <base-url>
# <packages-dir> holds the release .deb and .rpm. The signing key must already
# be in the GPG keyring; set MULSANNE_REPO_KEY to its fingerprint.
set -euo pipefail

packages=$1 out=$2 base_url=${3%/}
key=${MULSANNE_REPO_KEY:?set MULSANNE_REPO_KEY to the signing key fingerprint}
gpg_sign=(gpg --batch --yes --local-user "$key")

shopt -s nullglob
debs=("$packages"/*.deb) rpms=("$packages"/*.rpm)
shopt -u nullglob
if (( ${#debs[@]} == 0 || ${#rpms[@]} == 0 )); then
  echo "error: $packages needs at least one .deb and one .rpm" >&2
  exit 1
fi

rm -rf -- "$out"
mkdir -p -- "$out"
gpg --batch --export "$key" > "$out/mulsanne.gpg"
gpg --batch --armor --export "$key" > "$out/mulsanne.asc"

# apt: deb <base>/deb stable main
pool=$out/deb/pool/main/m/mulsanne
binary=$out/deb/dists/stable/main/binary-amd64
mkdir -p -- "$pool" "$binary"
cp -- "${debs[@]}" "$pool/"
(cd "$out/deb" && apt-ftparchive packages pool) > "$binary/Packages"
gzip -9kn -- "$binary/Packages"
apt-ftparchive \
  -o APT::FTPArchive::Release::Origin=Mulsanne \
  -o APT::FTPArchive::Release::Label=Mulsanne \
  -o APT::FTPArchive::Release::Suite=stable \
  -o APT::FTPArchive::Release::Codename=stable \
  -o APT::FTPArchive::Release::Architectures=amd64 \
  -o APT::FTPArchive::Release::Components=main \
  release "$out/deb/dists/stable" > "$out/Release.tmp"
mv -- "$out/Release.tmp" "$out/deb/dists/stable/Release"
"${gpg_sign[@]}" --clearsign -o "$out/deb/dists/stable/InRelease" "$out/deb/dists/stable/Release"
"${gpg_sign[@]}" --armor --detach-sign -o "$out/deb/dists/stable/Release.gpg" "$out/deb/dists/stable/Release"

# dnf/zypper: <base>/mulsanne.repo
mkdir -p -- "$out/rpm"
cp -- "${rpms[@]}" "$out/rpm/"
rpmsign --define "_gpg_name $key" --addsign "$out"/rpm/*.rpm
createrepo_c --no-database "$out/rpm"
"${gpg_sign[@]}" --armor --detach-sign -o "$out/rpm/repodata/repomd.xml.asc" "$out/rpm/repodata/repomd.xml"
cat > "$out/mulsanne.repo" <<EOF
[mulsanne]
name=Mulsanne
baseurl=$base_url/rpm
enabled=1
gpgcheck=1
repo_gpgcheck=1
gpgkey=$base_url/mulsanne.asc
EOF

cat > "$out/index.html" <<EOF
<!doctype html>
<meta charset="utf-8">
<title>Mulsanne Linux packages</title>
<p>Package repositories for <a href="https://mulsanne.xyz">Mulsanne</a>.
See the <a href="https://github.com/szpolny/mulsanne/blob/master/docs/linux.md">Linux installation guide</a>.</p>
EOF
