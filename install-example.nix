# Example NixOS configuration for installing Palet
# Add this to your NixOS configuration.nix or as a separate module

{ config, pkgs, ... }:

{
  # Import the flake (add this to your flake inputs first)
  imports = [
    # inputs.palet.nixosModules.default  # Add this line to imports
  ];

  # Enable and configure Palet
  programs.palet = {
    enable = true;
    
    # Optional: customize settings
    settings = {
      # Window configuration
      width = 800;
      height = 600;
      placeholder = "Search applications and commands...";
      
      # Terminal for tty commands
      terminal = "alacritty -e";  # or "xterm -e", "gnome-terminal --", etc.
      
      # Search URL for web fallback
      search_url = "https://duckduckgo.com/?q={q}";
      
      # Custom commands
      custom_commands = {
        # System commands
        htop = {
          name = "htop";
          command = "htop";
          description = "Interactive process viewer";
          tty = true;
        };
        
        shutdown = {
          name = "Shutdown";
          command = "systemctl poweroff";
          description = "Power off the system";
        };
        
        reboot = {
          name = "Restart";
          command = "systemctl reboot";
          description = "Restart the system";
        };
        
        suspend = {
          name = "Sleep";
          command = "systemctl suspend";
          description = "Suspend to RAM";
        };
        
        # Editor commands (with arguments)
        vim = {
          name = "vim";
          command = "vim";
          description = "Edit file with vim";
          accepts_arguments = true;
          tty = true;
        };
        
        code = {
          name = "code";
          command = "code";
          description = "Open with VS Code";
          accepts_arguments = true;
        };
        
        # Documentation
        man = {
          name = "man";
          command = "man";
          description = "Show manual page";
          accepts_arguments = true;
          tty = true;
        };
        
        # Network tools
        ping = {
          name = "ping";
          command = "ping";
          description = "Ping a host";
          accepts_arguments = true;
          tty = true;
        };
        
        # File operations
        find-file = {
          name = "find";
          command = "find . -name";
          description = "Find files by name";
          accepts_arguments = true;
          tty = true;
        };
      };
    };
  };
  
  # Optional: Add a keybinding to launch Palet
  # This example uses GNOME settings, adjust for your DE/WM
  # services.xserver.displayManager.gdm.enable = true;
  # services.xserver.desktopManager.gnome.enable = true;
  
  # Example keybinding configuration (adjust for your window manager)
  # environment.etc."palet-keybinding.desktop".text = ''
  #   [Desktop Entry]
  #   Type=Application
  #   Name=Palet Launcher
  #   Exec=palet
  #   NoDisplay=true
  # '';
}