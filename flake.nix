# {
#   description = "Dev shell the project";

#   inputs = {
#     fenix = {
#       url = "github:nix-community/fenix/monthly";
#       inputs.nixpkgs.follows = "nixpkgs";
#     };
#     nixpkgs.url = "nixpkgs/nixos-unstable";
#     flake-utils.url = "github:numtide/flake-utils";
#   };
#   outputs = { self, nixpkgs, flake-utils, fenix }:
#     flake-utils.lib.eachDefaultSystem (system:
#       let
#         pkgs = nixpkgs.legacyPackages.${system};
#         rust = fenix.packages.${system}.complete.toolchain;
#       in {
#         nixpkgs.overlays = [ fenix.overlays.complete ];
#         devShells.default = pkgs.mkShell rec {

#           nativeBuildInputs = with pkgs; [
#             pkg-config
#             cmake
#             pkg-config
#             freetype
#             expat
#             fontconfig
#           ];

#           buildInputs = [
#             rust
#             pkgs.lldb_15
#             pkgs.sccache
#             pkgs.udev
#             pkgs.alsa-lib
#             pkgs.vulkan-loader
#             pkgs.xorg.libX11
#             pkgs.xorg.libXcursor
#             pkgs.xorg.libXi
#             pkgs.xorg.libXrandr
#             pkgs.libxkbcommon
#             pkgs.wayland
#             pkgs.mold
#           ];
#           LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath buildInputs;
#           shellHook = "ln -s ${pkgs.jetbrains-mono}/share/fonts/truetype/JetBrainsMono-Regular.ttf ./fonts/";
#         };
#       });
# }

{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        craneLib = crane.lib.${system};
        pkgs = nixpkgs.legacyPackages.${system};

        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
          pkg-config
          freetype
          expat
          fontconfig
        ];

        buildInputs = [
          pkgs.udev
          pkgs.alsa-lib
          pkgs.vulkan-loader
          pkgs.xorg.libX11
          pkgs.xorg.libXcursor
          pkgs.xorg.libXi
          pkgs.xorg.libXrandr
          pkgs.libxkbcommon
          pkgs.wayland
          pkgs.mold
        ];

        graze = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          inherit nativeBuildInputs buildInputs;
        };

      in {
        packages.default = graze;
        devShells.default = pkgs.mkShell rec {
          LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath buildInputs;
          inherit nativeBuildInputs buildInputs;
          MAIN_FONT_PATH =
            "${pkgs.jetbrains-mono}/share/fonts/truetype/JetBrainsMono-Regular.ttf";
        };

        apps.default = graze;
      });
}

