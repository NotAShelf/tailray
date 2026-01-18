self: {
  config,
  pkgs,
  lib,
  ...
}: let
  inherit (lib.options) mkEnableOption mkPackageOption mkOption;
  inherit (lib.meta) getExe;
  inherit (lib.types) nullOr str enum;
  inherit (lib) mkIf optionals;

  cfg = config.services.tailray;
in {
  meta.maintainers = with lib.maintainers; [fufexan];

  options.services.tailray = {
    enable = mkEnableOption "Tailray, a Tailscale tray";

    package =
      mkPackageOption pkgs "tailray" {}
      // {
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
      type = enum ["light" "dark"];
      default = "light";
      example = "dark";
    };
  };

  config = mkIf cfg.enable {
    home.packages = [cfg.package];

    systemd.user.services.tailray = {
      Install.WantedBy = ["graphical-session.target"];

      Unit = {
        Description = "Tailscale tray item";
        Requires = "tray.target";
        After = ["graphical-session-pre.target" "tray.target"];
        PartOf = ["graphical-session.target"];
      };

      Service = {
        ExecStart = "${getExe cfg.package}";
        Restart = "always";
        RestartSec = "10";
        Environment =
          lib.optional (cfg.adminUrl != null)
          ["TAILRAY_ADMIN_URL=${cfg.adminUrl}"]
          ++ [
            "TAILRAY_THEME=${cfg.theme}"
          ];
      };
    };
  };
}
