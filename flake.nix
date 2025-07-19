{
  description = "Palet - A Rust GTK application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          rust-analyzer
          pkg-config
          wrapGAppsHook4
        ];

        buildInputs = with pkgs; [
          gtk4
          glib
          cairo
          pango
          gdk-pixbuf
          atk
          direnv
        ];

        runtimeDependencies = with pkgs; [
          gtk4
          glib
          gsettings-desktop-schemas
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          
          shellHook = ''
            export RUST_BACKTRACE=1
            export GSK_RENDERER=ngl
          '';

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeDependencies;
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "palet";
          version = "0.1.0";
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          inherit nativeBuildInputs buildInputs;
          
          # Ensure proper GTK app wrapping
          dontWrapGApps = false;
          
          # Install desktop entry
          postInstall = ''
            mkdir -p $out/share/applications
            cat > $out/share/applications/palet.desktop << EOF
            [Desktop Entry]
            Type=Application
            Name=Palet
            Comment=A fast application launcher
            Exec=$out/bin/palet
            Icon=application-x-executable
            Terminal=false
            Categories=Utility;
            Keywords=launcher;application;
            EOF
          '';
          
          meta = with pkgs.lib; {
            description = "A fast GTK4 application launcher with custom commands support";
            longDescription = ''
              Palet is a modern application launcher built with GTK4 and Rust.
              It provides fast application search, custom commands with argument support,
              arithmetic evaluation, and configurable styling.
            '';
            homepage = "https://github.com/user/palet";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.linux;
            mainProgram = "palet";
          };
        };
        
        # Make the package available as palet for easier reference
        packages.palet = self.packages.${system}.default;
      });
}
