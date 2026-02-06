{
  lib,
  mkShell,
  python3,
  pkg-config,
  rust-analyzer-unwrapped,
  rustfmt,
  clippy,
  cargo,
  rustc,
  lld,
  rustPlatform,
  # Build libs
  xorg,
  gtk3,
  libayatana-appindicator,
  libappindicator-gtk3,
  xdotool,
}:
mkShell {
  strictDeps = true;
  buildInputs = [
    xorg.libxcb
    xdotool
    gtk3
    libayatana-appindicator
    libappindicator-gtk3
  ];

  nativeBuildInputs = [
    pkg-config
    python3

    cargo
    rustc
    rust-analyzer-unwrapped
    (rustfmt.override {asNightly = true;})
    clippy
    lld
  ];

  env = {
    RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
    LD_LIBRARY_PATH = "${lib.makeLibraryPath [libayatana-appindicator libappindicator-gtk3 gtk3]}";
  };
}
