{ lib, pkgs, rust-bin, mkShell, packageName }:
mkShell {
  strictDeps = true;
  inputsFrom = [ pkgs.${packageName} ];
  packages = with pkgs; [
    # Derivations in `rust-stable` provide the toolchain,
    # must be listed first to take precedence over nightly.
    (rust-bin.stable.latest.minimal.override {
      extensions = [ "rust-src" "rust-docs" "clippy" ];
    })

    # Use rustfmt, and other tools that require nightly features.
    (rust-bin.selectLatestNightlyWith (toolchain:
      toolchain.minimal.override {
        extensions = [ "rustfmt" "rust-analyzer" ];
      }))

    cargo-insta
  ];
}
