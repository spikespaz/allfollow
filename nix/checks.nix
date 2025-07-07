{ self, lib, pkgs }:
let
  inherit (pkgs.stdenv.hostPlatform) system;

  packageChecks = lib.mapAttrs' (name: value: {
    name = "build-${name}";
    inherit value;
  }) self.packages.${system};

in packageChecks // {
  formatting-rust = pkgs.runCommandNoCCLocal "check-rust-formatting" {
    src = self;
    nativeBuildInputs = [
      (pkgs.rust-bin.selectLatestNightlyWith
        (toolchain: toolchain.minimal.override { extensions = [ "rustfmt" ]; }))
    ];
  } ''
    cd $src
    cargo fmt --check --message-format short
    touch $out
  '';

  formatting-nix = pkgs.runCommandNoCCLocal "check-nix-formatting" {
    src = self;
    nativeBuildInputs = [ pkgs.nixfmt-classic ];
  } ''
    cd $src
    nixfmt --check .
    touch $out
  '';
}
