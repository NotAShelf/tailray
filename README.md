# Tailray

A modern and fast implementation of tailscale-systray in Rust.

> [!NOTE] Tailray is a work in progress. Most things don't work, or work in a
> degraded state. If you find bugs that are not aggressed in the issues tab,
> feel free to create a new issue or a pull request! You are advi

## Hacking

Simply run `nix develop` in the project root.

## License

[tailscale-tray-rs]: https://github.com/dieterplex/tailscale-tray-rs
[upstream license]: https://github.com/dieterplex/tailscale-tray-rs/blob/60cfdec2942305085c2db295b56d8c666797e6ba/LICENSE

Tailray is based on, and is a aggressively refactored soft-fork of @dieterplex's
[tailscale-tray-rs] project and is licensed under the **MIT LICENSE** following
the [upstream license]. While much has changed, my thanks go to dieterplex for
their initial efforts that laid out an excellent foundation for Tailray.
