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
    home.packages = [ cfg.package ];

    xdg.configFile.limborsConfig = mkIf isDeclarativeConfig {
      target = "limbo-rs/config.yaml";
      source = configFile;
    };

    systemd.user.services.limbo-rs = {
      Unit = {
        Description = "limbo-rs system bar";
        Documentation = "https://github.com/co-conspirators/limbo-rs";
        PartOf = [ "graphical-session.target" ];
        After = [ "graphical-session.target" ];
        X-Restart-Triggers = mkIf isDeclarativeConfig [ configFile ];
      };
      Service = {
        ExecStart = "${cfg.package}/bin/limbo";
        Restart = "always";
        Environment = let
          libs = with pkgs; [ wayland libxkbcommon vulkan-loader libGL ];
          libPaths = lib.makeLibraryPath libs;
        in "LD_LIBRARY_PATH=${libPaths}";
      };
      Install.WantedBy = [ "graphical-session.target" ];
    };
  };
}
