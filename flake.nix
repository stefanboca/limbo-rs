{
  description = "Limbo bar, now with more rust";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ flake-parts, nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" ];

      perSystem = { config, self', inputs', pkgs, lib, system, ... }: {
        # use fenix overlay
        _module.args.pkgs = import nixpkgs {
          inherit system;
          overlays = [ inputs.fenix.overlays.default ];
        };

        packages = {
          limbo-rs = let
            inherit (inputs'.fenix.packages.minimal) toolchain;
            rustPlatform = pkgs.makeRustPlatform {
              cargo = toolchain;
              rustc = toolchain;
            };
          in rustPlatform.buildRustPackage {
            pname = "limbo-rs";
            version = "0-unstable";

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "Limbo bar, now with more rust";
              homepage = "https://github.com/co-conspirators/limbo-rs";
              license = lib.licenses.asl20;
              platforms = lib.platforms.linux;
              mainProgram = "limbo";
            };
          };

          default = self'.packages.limbo-rs;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = let
            dev = pkgs.writeShellApplication {
              name = "dev";
              runtimeInputs = with pkgs; [ cargo-watch ];
              text = "cargo-watch -c -w . -x run";
            };
          in with pkgs; [ cargo-watch dev ];

          shellHook = let
            libs = with pkgs; [ wayland libxkbcommon vulkan-loader libGL ];
            libPaths = lib.makeLibraryPath libs;
          in ''
            export LD_LIBRARY_PATH="${libPaths}:$LD_LIBRARY_PATH"
          '';
        };
      };
    };
}
