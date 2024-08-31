{ lib, pkgs, rust-bin, mkShell, packageName }:
let
  rust-stable = rust-bin.stable.latest.minimal.override {
    extensions = [ "rust-src" "rust-docs" "clippy" ];
  };
in mkShell {
  strictDeps = true;
  inputsFrom = [ pkgs.${packageName} ];
  packages = [
    # Derivations in `rust-stable` take precedence over nightly.
    (lib.hiPrio rust-stable)

    # Use rustfmt, and other tools that require nightly features.
    (rust-bin.selectLatestNightlyWith (toolchain:
      toolchain.minimal.override {
        extensions = [ "rustfmt" "rust-analyzer" ];
      }))
  ];
}
