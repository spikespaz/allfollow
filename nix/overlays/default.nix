{ self, lib, rust-overlay, packageName }: {
  default = lib.composeManyExtensions [
    # This is to ensure that `nix/overlays/package.nix` receives `rust-bin`.
    self.overlays.rust-overlay
    self.overlays.${packageName}
  ];
  # This flake exposes `overlays.rust-overlay` which is automatically applied
  # by `overlays.default`.
  rust-overlay = final: prev:
    if prev ? rust-bin then { } else rust-overlay.overlays.default final prev;
  # The main package name is defined in `flake.nix`.
  ${packageName} = import ./package.nix { sourceRoot = self; };
}
