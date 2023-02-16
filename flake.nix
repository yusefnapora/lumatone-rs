{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

 outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;
        inherit (pkgs.stdenv) isLinux;

        packages-linux = with pkgs; [
          pkg-config
          gtk3
          webkitgtk
          libayatana-appindicator.dev
          rustup
          alsa-lib.dev
        ];

        packages-darwin = with pkgs; [
          curl
          wget
          rustup
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