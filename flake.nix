{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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
          overlays = [ self.overlays.default ];
        });

      packageName = (lib.importTOML ./Cargo.toml).package.name;
    in {
      overlays =
        import ./nix/overlays { inherit self lib rust-overlay packageName; };

      packages = lib.mapAttrs (system: pkgs: {
        default = self.packages.${system}.${packageName};
        ${packageName} = pkgs.${packageName};
      }) pkgsFor;

      checks = eachSystem (system:
        lib.mapAttrs' (name: value: {
          name = "build-${name}";
          inherit value;
        }) self.packages.${system});

      devShells = lib.mapAttrs (system: pkgs: {
        default = pkgs.callPackage ./nix/shell.nix { inherit packageName; };
      }) pkgsFor;

      formatter =
        eachSystem (system: nixpkgs.legacyPackages.${system}.nixfmt-classic);
    };
}
