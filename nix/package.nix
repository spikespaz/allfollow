{ sourceRoot ? ../., lib, rustPlatform }:
let manifest = lib.importTOML "${sourceRoot}/Cargo.toml";
in rustPlatform.buildRustPackage {
  strictDeps = true;
  pname = manifest.package.name;
  version = manifest.package.version;
  cargoLock.lockFile = "${sourceRoot}/Cargo.lock";
  src = lib.cleanSource sourceRoot;
  meta = {
    inherit (manifest.package) description homepage;
    license = with lib.licenses; [ mit asl20 ];
    platforms = lib.platforms.unix;
    mainProgram = manifest.package.name;
  };
}
