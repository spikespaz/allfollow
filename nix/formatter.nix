{ nixfmt-classic, rust-bin, writeShellApplication, packageName }:
writeShellApplication {
  name = "${packageName}-treewide-formatter";
  runtimeInputs = [
    nixfmt-classic
    (rust-bin.selectLatestNightlyWith
      (toolchain: toolchain.minimal.override { extensions = [ "rustfmt" ]; }))
  ];
  text = ''
    bad_usage() {
      echo -e "\033[31merror:\033[0m $1" >&2
      echo -e "\033[33mUsage:\033[0m $(basename "$0") [ . | -- ]" >&2
      exit 1
    }

    if [ "$#" -gt 1 ]; then
      bad_usage "too many arguments"
    fi

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
        bad_usage "unknown argument '$src'"
        ;;
    esac
  '';
}
