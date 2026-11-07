{
  description = "Mulsanne — experimental Linux telemetry companion and HUD";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      mulsanne = pkgs.callPackage ./nix/package.nix { };
    in
    {
      packages.${system} = {
        default = mulsanne;
        inherit mulsanne;
      };
      apps.${system}.default = {
        type = "app";
        program = "${mulsanne}/bin/mulsanne";
      };
      checks.${system}.package = mulsanne;
    };
}
