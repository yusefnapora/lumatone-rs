{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system: 
    let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          # TODO: depend on rustc and cargo instead... bit fiddly tho
          # note that after installing standard toolchain, you also need to run
          # rustup target add wasm32-unknown-unknown
          rustup

          # native deps for tauri / dioxus
          webkitgtk
          gtk3
          libayatana-appindicator
          openssl.dev
          alsa-lib.dev
          pkg-config
        ];
      };

      # TODO: add package derivation for app, etc
    });
}