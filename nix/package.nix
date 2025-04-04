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

  junkfiles = [
    "flake.nix"
    "flake.lock"
    "LICENSE"
    ".gitignore"
    ".envrc"
    "README.md"
  ];

  repoDirFilter = name: type:
    !((type == "directory") && ((baseNameOf name) == "nix"))
    && !((type == "directory") && ((baseNameOf (dirOf name)) == ".github"))
    && !(builtins.any (r: (builtins.match r (baseNameOf name)) != null) junkfiles);

  cleanSource = src:
    lib.cleanSourceWith {
      filter = repoDirFilter;
      src = lib.cleanSource src;
    };
in
  stdenv.mkDerivation (finalAttrs: {
    pname = "tailray";
    version = "${cargoToml.package.version}-${rev}";

    src = cleanSource ../.;

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
