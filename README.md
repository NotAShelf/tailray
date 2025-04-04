<div align="center">
    <img src="https://deps.rs/repo/github/notashelf/tailray/status.svg" alt="https://deps.rs/repo/github/notashelf/tailray">
    <img src="https://img.shields.io/github/stars/notashelf/tailray?label=stars&color=DEA584">
</div>

<h1 align="center">Tailray</h1>

A modern and fast implementation of tailscale-systray in Rust. Redesigned from
ground up for a more maintainable codebase.

> [!NOTE]
> Tailray is a work in progress, but it is fully functional as far as usage
> goes. If things do not work, please do feel free to let me know. Contributions
> are also welcome, and so are feature requests.

## Features

- Status monitoring: Displays connection status through the tray icon
- IP management: Easy access to copy your IP or peer IPs to clipboard
- Privilege handling: Automatically elevates privileges when needed
- Desktop notifications: Receive notifications about connection status

## Usage

Tailray requires Tailscaled to be up and running. On Linux systems, you can
check its status with `systemctl status tailscaled`. After you confirm that
Tailscale is running, and that you are authenticated run `tailray` from a
terminal or consider writing a Systemd service for it.

Alternatively, start it directly with `tailray.` A Tailscale icon will appear in
your system tray. Click on it to access the menu:

- Connect/Disconnect: Toggle your Tailscale connection
- This device: View and copy your device's Tailscale IP address
- Network Devices: View and copy IP addresses of connected peer devices
  - My Devices: Personal devices on your Tailscale network
  - Tailscale Services: Service devices on your network
- Admin Console: Open the Tailscale admin web interface
- Exit Tailray: Close the application

### Overriding Admin Console URL

Tailray will assume `https://login.tailscale.com/admin/machines` to be the Admin
Console URL by default. You may override this URL by setting `TAILRAY_ADMIN_URL`
to an URL of your choice. This is useful if you are using Headscale as your
Tailscale coordination sever.

## Hacking

The recommended way of building Tailray is with the Nix build tool. You may run
`nix develop` in the repository to enter a devShell with the necessary
dependencies. Direnv users may also use `direnv allow` to let Direnv handle
their shell environment.

## License

[tailscale-tray-rs]: https://github.com/dieterplex/tailscale-tray-rs
[upstream license]: https://github.com/dieterplex/tailscale-tray-rs/blob/60cfdec2942305085c2db295b56d8c666797e6ba/LICENSE

Tailray is based on, and is a aggressively refactored soft-fork of @dieterplex's
[tailscale-tray-rs] project and is licensed under the **MIT LICENSE** following
the [upstream license]. While much has changed, my thanks go to dieterplex for
their initial efforts that laid out an excellent foundation for Tailray.
