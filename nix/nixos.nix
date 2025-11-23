{ config, options, lib, pkgs, ... }:

let
  inherit (lib) mkIf;

  cfg = config.services.limbo-rs;
  opt = options.services.limbo-rs;
in {
  options.services.limbo-rs = (import ./options.nix { inherit lib pkgs; });

  config = let
    yamlFormat = pkgs.formats.yaml { };
    isDeclarativeConfig = cfg.settings != opt.settings.default;
    configFile = if isDeclarativeConfig then
      yamlFormat.generate "config.yaml" cfg.settings
    else
      null;
  in mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    environment.etc.limborsConfig = mkIf isDeclarativeConfig {
      target = "limbo-rs/config.yaml";
      source = configFile;
    };

    systemd.user.services.limbo-rs = {
      description = "limbo-rs system bar";
      unitConfig.Documentation = "https://github.com/co-conspirators/limbo-rs";
      path = [ cfg.package ];
      after = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];
      wantedBy = [ "graphical-session.target" ];
      restartTriggers = mkIf isDeclarativeConfig [ configFile ];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/limbo";
        Restart = "always";
        Environment = let
          libs = with pkgs; [ wayland libxkbcommon vulkan-loader libGL ];
          libPaths = lib.makeLibraryPath libs;
        in "LD_LIBRARY_PATH=${libPaths}";
      };
    };
  };
}
