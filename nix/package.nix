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
  stdenv.mkDerivation {
    pname = "tailray";
    version = "${cargoToml.package.version}-${rev}";

    src = builtins.path {
      name = "tailray";
      path = ../.;
    };

    cargoDeps = rustPlatform.importCargoLock {
      lockFile = ../Cargo.lock;
      outputHashes = {
        "ksni-0.2.1" = "sha256-CKjOUGsqlMdgnNY6j29pP6S8wdZ73/v1dMyiIurlltI=";
      };
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
  }
