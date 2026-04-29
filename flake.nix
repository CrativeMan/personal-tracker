{
  description = "Personal tracker application";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
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

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        guiLibs = with pkgs; [
          libx11
          libxcursor
          libxrandr
          libxi
          libxinerama
          glew
          libGL
          wayland
          libxkbcommon
        ];

        commonNativeBuildInputs = with pkgs; [
          pkg-config
          cmake
        ];
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = [rustToolchain] ++ commonNativeBuildInputs;

          buildInputs = with pkgs;
            [
              openssl
              rust-analyzer
              lldb
              package-version-server
              python3
              ruff
              docker
              sqlx-cli
            ]
            ++ guiLibs;

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

        packages.default = rustPlatform.buildRustPackage {
          pname = "personal-tracker";
          version = "0.1.0";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = commonNativeBuildInputs;

          buildInputs = guiLibs;
        };
      }
    );
}
