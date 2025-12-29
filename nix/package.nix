{
  lib,
  dbus,
  pkg-config,
  rustPlatform,
  xorg,
  rev ? "dirty",
}: let
  cargoToml = lib.importTOML ../Cargo.toml;
in
  rustPlatform.buildRustPackage {
    pname = "tailray";
    version = "${cargoToml.package.version}-${rev}";

    src = lib.fileset.toSource {
      root = ../.;
      fileset = lib.fileset.unions [
        ../src
        ../Cargo.lock
        ../Cargo.toml
      ];
    };

    cargoLock.lockFile = ../Cargo.lock;

    strictDeps = true;
    nativeBuildInputs = [pkg-config];
    buildInputs = [dbus xorg.libxcb];

    meta = {
      description = "Rust implementation of tailscale-systray";
      homepage = "https://github.com/notashelf/tailray";
      license = lib.licenses.mit;
      mainProgram = "tailray";
      maintainers = with lib.maintainers; [NotAShelf];
    };
  }
