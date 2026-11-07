{
  lib,
  stdenv,
  stdenvNoCC,
  rustPlatform,
  pkgsCross,
  fetchPnpmDeps,
  pnpm,
  pnpmConfigHook,
  nodejs,
  pkg-config,
  wrapGAppsHook3,
  makeWrapper,
  gtk3,
  webkitgtk_4_1,
  openssl,
  bash,
  coreutils,
  util-linux,
}:
let
  # Keep launcher and documentation changes from recompiling the GUI.
  src = lib.cleanSourceWith {
    src = ../.;
    filter =
      path: type:
      !(lib.elem (baseNameOf path) [
        "target"
        "node_modules"
        "dist"
        "docs"
        "tests"
        ".github"
        "nix"
        "flake.nix"
        "flake.lock"
        "packaging"
        "scripts"
      ])
      && lib.cleanSourceFilter path type;
  };
  version = (lib.importJSON ../package.json).version;
  frontend = stdenv.mkDerivation {
    pname = "mulsanne-frontend";
    inherit src version;
    nativeBuildInputs = [
      nodejs
      pnpm
      pnpmConfigHook
    ];
    pnpmDeps = fetchPnpmDeps {
      pname = "mulsanne";
      inherit src pnpm;
      fetcherVersion = 5;
      hash = "sha256-Rp72wuidaNcuOb1zL4shjZ34dKUxDVrT274VSrWKd0k=";
    };
    buildPhase = "pnpm build";
    installPhase = "cp -r dist $out";
  };
  bridge = pkgsCross.mingwW64.rustPlatform.buildRustPackage {
    pname = "mulsanne-lmu-bridge";
    inherit src version;
    cargoRoot = "lmu-bridge";
    buildAndTestSubdir = "lmu-bridge";
    cargoLock.lockFile = ../lmu-bridge/Cargo.lock;
    doCheck = false; # Lifecycle tests run on Windows in CI.
    installPhase = ''
      install -Dm755 target/x86_64-pc-windows-gnu/release/lmu-bridge.exe $out/bin/lmu-bridge.exe
    '';
  };
  ui = rustPlatform.buildRustPackage {
    pname = "mulsanne-ui";
    inherit src version;
    cargoRoot = "src-tauri";
    buildAndTestSubdir = "src-tauri";
    cargoLock.lockFile = ../src-tauri/Cargo.lock;
    buildFeatures = [ "tauri/custom-protocol" ];
    nativeBuildInputs = [
      pkg-config
      wrapGAppsHook3
    ];
    buildInputs = [
      gtk3
      webkitgtk_4_1
      openssl
    ];
    env.TAURI_CONFIG = builtins.toJSON { build.frontendDist = "${frontend}"; };
    installPhase = ''
      install -Dm755 target/x86_64-unknown-linux-gnu/release/lm-overlay $out/bin/lm-overlay
    '';
  };
in
stdenvNoCC.mkDerivation {
  pname = "mulsanne";
  inherit version;
  dontUnpack = true;
  nativeBuildInputs = [ makeWrapper ];
  installPhase = ''
    mkdir -p $out/bin
    ln -s ${ui}/bin/lm-overlay $out/bin/lm-overlay
    install -Dm755 ${../scripts/run-lmu-bridge.sh} $out/lib/mulsanne/run-lmu-bridge.sh
    install -Dm644 ${../packaging/linux/hyprland.lua} $out/lib/mulsanne/hyprland.lua
    install -Dm644 ${bridge}/bin/lmu-bridge.exe $out/lib/mulsanne/lmu-bridge.exe
    install -Dm644 ${../packaging/linux/mulsanne.desktop} $out/share/applications/mulsanne.desktop
    install -Dm644 ${../src-tauri/icons/128x128.png} $out/share/icons/hicolor/128x128/apps/lm-overlay.png
    makeWrapper ${bash}/bin/bash $out/bin/mulsanne \
      --add-flags ${../packaging/linux/launch.sh} \
      --set MULSANNE_LIBEXEC $out/lib/mulsanne \
      --set MULSANNE_BINARY ${ui}/bin/lm-overlay \
      --prefix PATH : ${
        lib.makeBinPath [
          bash
          coreutils
          util-linux
        ]
      }
  '';
  passthru = { inherit frontend bridge ui; };
  meta = {
    description = "Experimental Linux HUD and telemetry companion for Le Mans Ultimate";
    homepage = "https://mulsanne.xyz";
    license = lib.licenses.mit;
    platforms = [ "x86_64-linux" ];
    mainProgram = "mulsanne";
  };
}
