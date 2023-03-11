{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

 outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
    let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        inherit (pkgs) lib;
        inherit (pkgs.stdenv) isLinux;

        rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = ["wasm32-unknown-unknown"];
        };

        packages-linux = with pkgs; [
          rust-toolchain
          pkg-config
          gtk3
          webkitgtk
          libayatana-appindicator.dev
          alsa-lib.dev
        ];

        packages-darwin = with pkgs; [
          rust-toolchain
          curl
          wget
          pkg-config
          libiconv
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.CoreServices
          darwin.apple_sdk.frameworks.CoreFoundation
          darwin.apple_sdk.frameworks.Foundation
          darwin.apple_sdk.frameworks.AppKit
          darwin.apple_sdk.frameworks.WebKit
          darwin.apple_sdk.frameworks.Cocoa
        ];

        packages = if isLinux then packages-linux else packages-darwin;
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = packages;
        };
      });
}
