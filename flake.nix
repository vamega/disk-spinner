{
  nixConfig = {
    extra-substituters = [
      "https://disk-spinner.cachix.org"
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "disk-spinner.cachix.org-1:kZjfVwFLafd0w0D2yAPvfKiXZSCy+Y2ittfJ0pwiYKs="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  outputs = inputs @ {
    self,
    flake-parts,
    nixpkgs,
    fenix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      imports = [
        inputs.devshell.flakeModule
        inputs.flake-parts.flakeModules.easyOverlay
      ];

      perSystem = {
        config,
        pkgs,
        final,
        system,
        ...
      }: {
        formatter = pkgs.alejandra;

        packages.default = config.packages.disk-spinner;
        packages.disk-spinner = let
          rustPlatform = pkgs.makeRustPlatform {
            inherit (fenix.packages.${system}.stable) rustc cargo;
          };
        in
          rustPlatform.buildRustPackage {
            pname = "disk-spinner";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
            src = let
              fs = pkgs.lib.fileset;
            in
              fs.toSource {
                root = ./.;
                fileset = fs.unions [
                  ./Cargo.toml
                  ./Cargo.lock
                  ./src
                ];
              };

            doCheck = false; # The sandbox blocks io_uring, which makes testing this program impossible.
            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "shishua-0.2.0" = "sha256-jP8+tWuXxISk4MFco8a0HWTQz6dC9UvVnufqR15EIYw=";
              };
            };
            meta.mainProgram = "disk-spinner";
          };

        apps = {
          default = config.apps.disk-spinner;
          disk-spinner.program = config.packages.disk-spinner;
        };

        devshells = {
          default = {
            imports = [
              "${inputs.devshell}/extra/language/rust.nix"
            ];
            packages = [fenix.packages.${system}.stable.rust-analyzer];
            language.rust = {
              enableDefaultToolchain = false;
              packageSet = fenix.packages.${system}.stable;
              tools = [
                "rust-analyzer"
                "cargo"
                "clippy"
                "rustfmt"
                "rustc"
              ];
            };
          };
        };

        overlayAttrs = {inherit (config.packages) disk-spinner;};
      };
    };

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
