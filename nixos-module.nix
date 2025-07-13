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
    
    settings = mkOption {
      type = types.attrs;
      default = {};
      description = ''
        Configuration for Palet. This will be written to the system-wide config file.
        Users can override this with their own ~/.config/palet/config.toml file.
      '';
      example = literalExpression ''
        {
          width = 800;
          height = 600;
          placeholder = "Search applications...";
          terminal = "alacritty -e";
          search_url = "https://duckduckgo.com/?q={q}";
          
          custom_commands = {
            htop = {
              name = "htop";
              command = "htop";
              description = "System monitor";
              tty = true;
            };
            
            shutdown = {
              name = "Shutdown";
              command = "systemctl poweroff";
              description = "Power off the system";
            };
          };
        }
      '';
    };
  };
  
  config = mkIf cfg.enable {
    # Install the package system-wide
    environment.systemPackages = [ paletPackage ];
    
    # Create system-wide config if settings are provided
    environment.etc."palet/config.toml" = mkIf (cfg.settings != {}) {
      text = generators.toTOML {} cfg.settings;
      mode = "0644";
    };
    
    # Ensure required dependencies are available
    programs.dconf.enable = mkDefault true;
    
    # Add to PATH for all users
    environment.variables = {
      PATH = [ "${paletPackage}/bin" ];
    };
    
    # Optional: Create a systemd user service for auto-start
    # systemd.user.services.palet = mkIf cfg.autoStart {
    #   description = "Palet Application Launcher";
    #   wantedBy = [ "graphical-session.target" ];
    #   serviceConfig = {
    #     Type = "simple";
    #     ExecStart = "${paletPackage}/bin/palet --daemon";
    #     Restart = "on-failure";
    #   };
    # };
  };
  
  meta = {
    maintainers = with maintainers; [ ];
    doc = ./README.md;
  };
}