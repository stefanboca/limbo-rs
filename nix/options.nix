{ lib, pkgs, ... }:

let
  inherit (lib) mkOption mkEnableOption types;
  yamlFormat = pkgs.formats.yaml { };
in {
  enable = mkEnableOption "Enable limbo-rs bar";

  package = mkOption {
    type = types.package;
    defaultText = "pkgs.limbo-rs";
    description = "The package to use for limbo-rs.";
  };

  settings = mkOption {
    type = yamlFormat.type;
    default = null;
    description = "Settings for the limbo-rs package, YAML format.";
  };
}
