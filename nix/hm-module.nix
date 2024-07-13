self: {
  config,
  pkgs,
  lib,
  ...
}: let
  cfg = config.services.tailray;

  inherit (lib.meta) getExe;
  inherit (lib.options) mkEnableOption mkPackageOption;
in {
  meta.maintainers = with lib.maintainers; [fufexan];

  options.services.tailray = {
    enable = mkEnableOption "Tailray, a Tailscale tray";

    package =
      mkPackageOption pkgs "tailray" {}
      // {
        default = self.packages.${pkgs.system}.default;
      };
  };

  config = lib.mkIf cfg.enable {
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
      };
    };
  };
}
