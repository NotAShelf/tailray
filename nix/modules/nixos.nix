self:
{ config, pkgs, lib, ... }:
let
  inherit (lib.options) mkEnableOption mkPackageOption mkOption;
  inherit (lib.meta) getExe;
  inherit (lib.types) nullOr str enum;
  inherit (lib) mkIf;

  cfg = config.services.tailray;
in {
  meta.maintainers = with lib.maintainers; [ NotAShelf ];

  options.services.tailray = {
    enable = mkEnableOption "Tailray, a Tailscale tray";

    package = mkPackageOption pkgs "tailray" { } // {
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.tailray;
    };

    adminUrl = mkOption {
      description = "The URL the Admin Console button should point to";
      type = nullOr str;
      default = null;
      example = "https://headplane.example.com/admin/login";
    };

    theme = mkOption {
      description = "Icon Theme";
      type = enum [ "light" "dark" ];
      default = "light";
      example = "dark";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    systemd.services.tailray = {
      serviceConfig = {
        WantedBy = [ "multi-user.target" ];
        ExecStart = "${getExe cfg.package}";
        Restart = "always";
        RestartSec = "10";
      };

      environment = {
        TAILRAY_ADMIN_URL = mkIf (cfg.adminUrl != null) cfg.adminUrl;
        TAILRAY_THEME = {cfg.theme};
      };
    };
  };
}
