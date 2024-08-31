{ sourceRoot }:
final: _prev:
let
  rust-stable = final.rust-bin.stable.latest.minimal;
  rustPlatform = final.makeRustPlatform {
    cargo = rust-stable;
    rustc = rust-stable;
  };
  manifest = builtins.fromTOML (builtins.readFile "${sourceRoot}/Cargo.toml");
in {
  ${manifest.package.name} =
    final.callPackage ../package.nix { inherit sourceRoot rustPlatform; };
}
