{
  description = "Adabraka-UI Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Provides the latest Rust toolchains
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        runtimeDeps = with pkgs; [
          vulkan-loader
          libxkbcommon
          wayland
          libX11
          libXcursor
          libXi
          libxcb      # Added for X11 connection support
          alsa-lib
        ];

        buildDeps = with pkgs; [
          pkg-config
          fontconfig
          freetype
          openssl
          dbus
          protobuf         # Added for protoc
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
            pkgs.pkg-config
            pkgs.protobuf  # Also here for the binary
          ];

          buildInputs = buildDeps ++ runtimeDeps;

          # EXPLICIT ENVIRONMENT VARIABLES
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeDeps;
          PROTOC = "${pkgs.protobuf}/bin/protoc"; # Tells cargo where protoc is
          PROTOC_INCLUDE = "${pkgs.protobuf}/include";

          shellHook = ''
            export PATH="$HOME/.cargo/bin:$PATH"
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeDeps}:$LD_LIBRARY_PATH
            # This helps GPUI detect the platform
            export XDG_SESSION_TYPE=wayland
            echo "--- Adabraka-UI + Retiscope Dev Environment ---"
            echo "Protoc: $(protoc --version)"
          '';
        };
  });
}
