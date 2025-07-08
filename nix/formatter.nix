{ nixfmt-classic, rust-bin, writeShellApplication, packageName }:
writeShellApplication {
  name = "${packageName}-treewide-formatter";
  runtimeInputs = [
    nixfmt-classic
    (rust-bin.selectLatestNightlyWith
      (toolchain: toolchain.minimal.override { extensions = [ "rustfmt" ]; }))
  ];
  text = ''
    src="''${1:-.}"
    case "$src" in
      --)
        nixfmt --
        ;;
      .)
        nixfmt .
        cargo fmt
        ;;
      *)
        echo "Usage: $(basename "$0") [ . | -- ]" >&2
        exit 1
        ;;
    esac
  '';
}
