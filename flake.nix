{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
              rust-analyzer
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }) // {
        nixosModule = { config, lib, pkgs, ... }:
          with lib;
          let cfg = config.services.url-eater;
          in {
            options.services.url-eater = {
              enable = mkEnableOption "Enables the URL Eater service";
              filters = mkOption {
                type = types.str;
                example = ''
                  category "Spotify" {
                  	params "context@open.spotify.com" "si@open.spotify.com"
                  }
                  category "Twitter" {
                  	params "cxt@*.twitter.com" "ref_*@*.twitter.com" "s@*.twitter.com" "t@*.twitter.com" "twclid"
                  }
                '';
                description = mdDoc ''
                  A list of filters to use, in the KDL file format.
                '';
              };
            };

            config = mkIf cfg.enable {
              systemd.user.services."url-eater" = let
                filters = pkgs.writeText "filters.kdl" cfg.filters;
                pkg = self.defaultPackage.${pkgs.system};
              in {
                description = "Clipboard URL cleanup service";

                after = [ "graphical-session-pre.target" ];
                wantedBy = [ "graphical-session.target" ];

                script = ''
                  exec ${pkg}/bin/url-eater ${filters}
                '';
              };
            };
          };
      };
}
