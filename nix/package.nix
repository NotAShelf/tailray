{
  lib,
  stdenv,
  cargo,
  dbus,
  meson,
  ninja,
  python3,
  pkg-config,
  rustc,
  rustPlatform,
  xorg,
  rev ? "dirty",
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
  stdenv.mkDerivation (finalAttrs: {
    pname = "tailray";
    version = "${cargoToml.package.version}-${rev}";

    src = lib.fileset.toSource {
      root = ../.;
      fileset = lib.fileset.unions [
        ../icons
        ../src
        ../Cargo.lock
        ../Cargo.toml

        ../meson.build
      ];
    };

    cargoDeps = rustPlatform.importCargoLock {
      lockFile = "${finalAttrs.src}/Cargo.lock";
    };

    strictDeps = true;

    nativeBuildInputs = [
      meson
      ninja
      pkg-config
      rustPlatform.cargoSetupHook
      cargo
      rustc
      python3
    ];

    buildInputs = [
      dbus
      xorg.libxcb
    ];

    meta = {
      description = "Rust implementation of tailscale-systray";
      homepage = "https://github.com/notashelf/tailray";
      license = lib.licenses.mit;
      mainProgram = "tailray";
      maintainers = with lib.maintainers; [NotAShelf];
    };
  })
