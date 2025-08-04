self: {
  config,
  pkgs,
  lib,
  ...
}: let
  inherit (lib.options) mkEnableOption mkPackageOption;
  inherit (lib.meta) getExe;

  cfg = config.services.tailray;
in {
  meta.maintainers = with lib.maintainers; [fufexan];

  options.services.tailray = {
    enable = mkEnableOption "Tailray, a Tailscale tray";

    package =
      mkPackageOption pkgs "tailray" {}
      // {
        default = self.packages.${pkgs.system}.tailray;
      };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [cfg.package];

    systemd.services.tailray = {
      serviceConfig = {
        WantedBy = ["multi-user.target"];
        ExecStart = "${getExe cfg.package}";
        Restart = "always";
        RestartSec = "10";
      };
    };
  };
}
