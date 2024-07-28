{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

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
      default = import ./nix/hm-module.nix self;
    };
  };
}
