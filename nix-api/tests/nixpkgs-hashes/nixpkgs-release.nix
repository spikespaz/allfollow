let
  sources = import ./npins/default.nix;
  inherit (sources) nixpkgs;
in import "${nixpkgs}/pkgs/top-level/release-outpaths.nix" {
  checkMeta = false;
  attrNamesOnly = true;
  systems = null;
}
