{ withSystem, ... }:

{
  flake = {
    nixosModules.default = { pkgs, ... }: {
      imports = [ ./nixos.nix ];
      services.limbo-rs.package = withSystem pkgs.stdenv.hostPlatform.system
        ({ config, ... }: config.packages.default);
    };

    homeManagerModules.default = { pkgs, ... }: {
      imports = [ ./home-manager.nix ];
      services.limbo-rs.package = withSystem pkgs.stdenv.hostPlatform.system
        ({ config, ... }: config.packages.default);
    };
  };
}
