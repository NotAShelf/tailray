{
  description = "Small Tailscale Tray Manager";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";

  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    systems = ["x86_64-linux" "aarch64-linux"];
    forEachSystem = nixpkgs.lib.genAttrs systems;
    pkgsForEach = nixpkgs.legacyPackages;
  in {
    devShells = forEachSystem (system: {
      default = pkgsForEach.${system}.callPackage ./nix/shell.nix {};
    });

    packages = forEachSystem (system: {
      tailray = pkgsForEach.${system}.callPackage ./nix/package.nix {};
      default = self.packages.${system}.tailray;
    });

    homeManagerModules = {
      tailray = import ./nix/modules/home-manager.nix self;
      default = self.homeManagerModules.tailray;
    };

    nixosModules = {
      tailray = import ./nix/modules/nixos.nix self;
      default = self.nixosModules.tailray;
    };
  };
}
