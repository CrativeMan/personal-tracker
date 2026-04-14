{
  description = "Rust Rover environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        rustToolchain = pkgs.rust-bin.stable."1.94.0".default.override {
          extensions = ["rust-src" "clippy" "rustfmt"];
        };
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
            pkg-config
            rust-analyzer
            lldb
            package-version-server
            python3
            ruff

            libx11
            libxcursor
            libxrandr
            libxi
            libxinerama
            glew
            libGL
            cmake
            wayland-scanner
            wayland
            wayland-protocols
            libxkbcommon

            docker
            sqlx-cli
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.libGL
            pkgs.wayland
            pkgs.libxkbcommon
            pkgs.libxcursor
            pkgs.libx11
            pkgs.libxrandr
            pkgs.libxi
          ];

          shellHook = ''
            mkdir -p ~/.rust-rover/toolchain

            ln -sfn ${rustToolchain}/lib ~/.rust-rover/toolchain
            ln -sfn ${rustToolchain}/bin ~/.rust-rover/toolchain

            export RUST_SRC_PATH="$HOME/.rust-rover/toolchain/lib/rustlib/src/rust/library"
          '';
        };
      }
    );
}
