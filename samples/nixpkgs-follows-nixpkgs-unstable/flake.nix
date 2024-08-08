{
  inputs = {
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    nixpkgs-release.url = "github:nixos/nixpkgs/release-24.05";
    nixpkgs.follows = "nixpkgs-unstable";
  };
  outputs = { ... }: { };
}
