{
  mkShell,
  dbus,
  python3,
  pkg-config,
  rust-analyzer-unwrapped,
  rustfmt,
  clippy,
  cargo,
  rustc,
  rustPlatform,
  libxcb,
}:
mkShell {
  env."RUST_SRC_PATH" = "${rustPlatform.rustLibSrc}";

  strictDeps = true;
  buildInputs = [dbus libxcb];
  nativeBuildInputs = [
    pkg-config
    python3

    cargo
    rustc
    rust-analyzer-unwrapped
    (rustfmt.override {asNightly = true;})
    clippy
  ];
}
