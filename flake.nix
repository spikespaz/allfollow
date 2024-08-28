{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
  };
  outputs = { self, nixpkgs, rust-overlay, systems }:
    let
      inherit (nixpkgs) lib;
      eachSystem = lib.genAttrs (import systems);
      pkgsFor = eachSystem (system:
        import nixpkgs {
          localSystem = system;
          overlays = [ self.overlays.default rust-overlay.overlays.default ];
        });
    in {
      overlays = {
        default = self.overlays.allfollow;
        allfollow = pkgs: pkgs0:
          let
            rust-bin = rust-overlay.lib.mkRustBin { } pkgs;
            rust-stable = rust-bin.stable.latest.minimal;
            rustPlatform = pkgs.makeRustPlatform {
              cargo = rust-stable;
              rustc = rust-stable;
            };
          in {
            allfollow =
              pkgs.callPackage ./nix/default.nix { inherit rustPlatform; };
          };
      };

      packages = eachSystem (system: {
        default = self.packages.${system}.allfollow;
        inherit (pkgsFor.${system}) allfollow;
      });

      devShells = eachSystem (system:
        let
          pkgs = pkgsFor.${system};
          rust-stable = pkgs.rust-bin.stable.latest.minimal.override {
            extensions = [ "rust-src" "rust-docs" "clippy" ];
          };
        in {
          default = pkgs.mkShell {
            strictDeps = true;
            packages = with pkgs; [
              # Derivations in `rust-stable` take precedence over nightly.
              (lib.hiPrio rust-stable)

              # Use rustfmt, and other tools that require nightly features.
              (rust-bin.selectLatestNightlyWith (toolchain:
                toolchain.minimal.override {
                  extensions = [ "rustfmt" "rust-analyzer" ];
                }))
            ];
            # RUST_BACKTRACE = 1;
          };
        });

      formatter = eachSystem (system: pkgsFor.${system}.nixfmt-classic);
    };
}
