{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = (pkgs.extend self.overlays.default).url-eater;

        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              cargo-audit
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
              rust-analyzer
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    )
    // {
      overlays.default =
        final: _:
        let
          naersk-lib = final.callPackage naersk { };
          withMeta = final.lib.addMetaAttrs { mainProgram = "url-eater"; };
        in
        {
          url-eater = withMeta (
            naersk-lib.buildPackage {
              src = ./.;
              buildInputs = with final; lib.optionals stdenv.hostPlatform.isDarwin [ apple-sdk ];
            }
          );
        };

      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        with lib;
        let
          cfg = config.services.url-eater;
        in
        {
          options.services.url-eater = {
            enable = mkEnableOption "Enables the URL Eater service";
            package = mkPackageOption pkgs "url-eater" { };
            filters = mkOption {
              type = types.str;
              example = /* kdl */ ''
                category "Spotify" {
                	params "context@open.spotify.com" "si@open.spotify.com"
                }
                category "Twitter" {
                	params "cxt@*.twitter.com" "ref_*@*.twitter.com" "s@*.twitter.com" "t@*.twitter.com" "twclid"
                }
              '';
              description = ''
                A list of filters to use, in the KDL file format.
              '';
            };
          };

          config = mkIf cfg.enable {
            nixpkgs.overlays = lib.mkBefore [ self.overlays.default ];

            systemd.user.services."url-eater" =
              let
                filters = pkgs.writeText "filters.kdl" cfg.filters;
              in
              {
                description = "Clipboard URL cleanup service";

                after = [ "graphical-session-pre.target" ];
                wantedBy = [ "graphical-session.target" ];

                script = ''
                  exec ${lib.getExe cfg.package.url-eater} ${filters}
                '';
              };
          };
        };
    };
}
