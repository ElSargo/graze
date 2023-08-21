{
  description = "Dev shell the project";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rust = fenix.packages.${system}.complete.toolchain;
      in {
        nixpkgs.overlays = [ fenix.overlays.complete ];
        devShells.default = pkgs.mkShell rec {

          nativeBuildInputs = with pkgs; [
            pkg-config
            cmake
            pkg-config
            freetype
            expat
            fontconfig
          ];

          buildInputs = [
            rust
            pkgs.lldb_15
            pkgs.sccache
            pkgs.udev
            pkgs.alsa-lib
            pkgs.vulkan-loader
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libXrandr
            pkgs.libxkbcommon
            pkgs.wayland
          ];
          LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath buildInputs;
          shellHook = "ln -s ${pkgs.jetbrains-mono}/share/fonts/truetype/JetBrainsMono-Regular.ttf ./fonts/";
        };
      });
}

