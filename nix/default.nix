{
# Must be provided in `callPackage` for accuracy.
sourceRoot ? ./..,
#
lib, rustPlatform
#
}:
let manifest = lib.importTOML "${sourceRoot}/Cargo.toml";
in rustPlatform.buildRustPackage {
  pname = manifest.package.name;
  version = manifest.package.version;
  src = lib.cleanSource sourceRoot;
  cargoLock.lockFile = "${sourceRoot}/Cargo.lock";
  meta = {
    inherit (manifest.package) description homepage;
    license = lib.licenses.mit;
    maintainers = [ lib.maintainers.spikespaz ];
    platforms = lib.platforms.linux ++ lib.platforms.darwin;
    mainProgram = manifest.package.name;
  };
}
