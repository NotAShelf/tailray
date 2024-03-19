{
  lib,
  dbus,
  python3,
  pkg-config,
  rustPlatform,
  xorg,
  rev ? "dirty",
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
  rustPlatform.buildRustPackage {
    pname = "tailray";
    version = "${cargoToml.package.version}-${rev}";

    src = lib.fileset.toSource {
      root = ../.;
      fileset =
        lib.fileset.intersection
        (lib.fileset.fromSource (lib.sources.cleanSource ../.))
        (lib.fileset.unions [
          ../src
          ../Cargo.toml
          ../Cargo.lock
        ]);
    };

    cargoLock.lockFile = ../Cargo.lock;

    strictDeps = true;

    buildInputs = [dbus xorg.libxcb];
    nativeBuildInputs = [
      pkg-config
      python3
    ];

    meta = {
      description = "Rust implementation of tailscale-systray";
      homepage = "https://github.com/notashelf/tailray";
      license = lib.licenses.gpl3Plus;
      mainProgram = "tailray";
      maintainers = with lib.maintainers; [NotAShelf];
    };
  }
