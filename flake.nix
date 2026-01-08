{
  description = "Small Tailscale Tray Manager";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";

  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    # We cannot actually support all systems covered in flakeExposed
    # but I *really* doubt we'll get anyone running anything other than
    # Linux and Darwin, or very rarely BSD. If those people come to my
    # issue tracker, I might as well try to support those platforms.
    # Who the hell uses powerpc in 2026?
    systems = nixpkgs.lib.systems.flakeExposed;
    forEachSystem = nixpkgs.lib.attrsets.genAttrs systems;
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
