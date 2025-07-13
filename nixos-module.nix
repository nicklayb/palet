{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.palet;
  
  paletPackage = if cfg.package != null then cfg.package else
    (import ./flake.nix).outputs.packages.${pkgs.system}.default;
    
in {
  options.programs.palet = {
    enable = mkEnableOption "Palet application launcher";
    
    package = mkOption {
      type = types.nullOr types.package;
      default = null;
      description = ''
        Package to use for Palet. If null, uses the default package from the flake.
      '';
    };
  };
  
  config = mkIf cfg.enable {
    environment.systemPackages = [ paletPackage ];
  };
  
  meta = {
    maintainers = with maintainers; [ ];
    doc = ./README.md;
  };
}
