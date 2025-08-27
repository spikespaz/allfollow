let
  sources = import ./npins/default.nix;
  inherit (sources) nixpkgs;
in import "${nixpkgs}/pkgs/top-level/release.nix" {
  inherit nixpkgs;
  nixpkgsArgs = {
    config = {
      allowAliases = false;
      allowUnfree = true;
      inHydra = true; # unsure
    };
    __allowFileset = false; # unsure
  };
}
