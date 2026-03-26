{
  description = "zr — relocate directories while preserving zoxide scores";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      imports = [
        inputs.treefmt-nix.flakeModule
      ];

      perSystem = {
        pkgs,
        system,
        config,
        ...
      }: let
        overlays = [(import inputs.rust-overlay)];
        rustPkgs = import inputs.nixpkgs {inherit system overlays;};
        rustToolchain = rustPkgs.rust-bin.stable.latest.default;
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource ./.;
        commonArgs = {
          inherit src;
          pname = "zr";
          version = "0.1.0";
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        zr = craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts;
            postInstall = ''
              install -Dm644 ${./completions/zr.fish} $out/share/fish/vendor_completions.d/zr.fish
              install -Dm644 ${./completions/zr.bash} $out/share/bash-completion/completions/zr
              install -Dm644 ${./completions/_zr} $out/share/zsh/site-functions/_zr
            '';
          });
      in {
        packages = {
          default = zr;
          zr = zr;
        };

        treefmt = {
          projectRootFile = "flake.nix";
          programs = {
            alejandra.enable = true;
            rustfmt.enable = true;
          };
        };

        devShells.default = craneLib.devShell {
          packages = [
            pkgs.cargo-watch
            config.treefmt.build.wrapper
          ];
          inputsFrom = [zr];
        };
      };
    };
}
