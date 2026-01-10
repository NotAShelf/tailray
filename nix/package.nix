{
  lib,
  pkg-config,
  rustPlatform,
  xorg,
  gtk3,
  libayatana-appindicator,
  libappindicator-gtk3,
  makeWrapper,
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
    nativeBuildInputs = [pkg-config makeWrapper];
    buildInputs = [
      xorg.libxcb
      gtk3
      libayatana-appindicator
      libappindicator-gtk3
    ];

    # Wrap the binary to set LD_LIBRARY_PATH for runtime library loading
    postFixup = ''
      wrapProgram $out/bin/tailray \
        --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [
        libayatana-appindicator
        libappindicator-gtk3
        gtk3
      ]}
    '';

    meta = {
      description = "Rust implementation of tailscale-systray";
      homepage = "https://github.com/notashelf/tailray";
      license = lib.licenses.mit;
      mainProgram = "tailray";
      maintainers = with lib.maintainers; [NotAShelf];
    };
  }
