# Tailray

A modern and fast implementation of tailscale-systray in Rust.

<!-- deno-fmt-ignore-start -->

## Usage

> [!NOTE]
> Tailray is a work in progress. Most things don't work, or work in a
> degraded state. If you find bugs that are not addressed in the issues tab,
> feel free to create a new issue or a pull request!

<!-- deno-fmt-ignore-end -->

Tailray requires Tailscaled to be up and running. On Linux systems, you can
check its status with `systemctl status tailscaled`.

After you confirm that Tailscale is running, and that you are authenticated run
`tailray` from a terminal or consider writing a systemd service for it.

### Overriding Admin Console URL

Tailray will assume `https://login.tailscale.com/admin/machines` to be the Admin
Console URL by default. You may override this URL by setting `TAILRAY_ADMIN_URL`
to an URL of your choice.

## Hacking

The recommended way of building Tailray is with the Nix build tool. You may run
`nix develop` in the repository to enter a devShell with the necessary
dependencies. Direnv users may also use `direnv allow` to let direnv handle
their shell environment.

## License

[tailscale-tray-rs]: https://github.com/dieterplex/tailscale-tray-rs
[upstream license]: https://github.com/dieterplex/tailscale-tray-rs/blob/60cfdec2942305085c2db295b56d8c666797e6ba/LICENSE

Tailray is based on, and is a aggressively refactored soft-fork of @dieterplex's
[tailscale-tray-rs] project and is licensed under the **MIT LICENSE** following
the [upstream license]. While much has changed, my thanks go to dieterplex for
their initial efforts that laid out an excellent foundation for Tailray.
