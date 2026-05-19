{
  description = "ASAR archive management CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};

        asar-cli = pkgs.rustPlatform.buildRustPackage {
          pname = "asar-cli";
          version = "0.1.0";

          src = ./.;

          buildAndTestSubdir = "cli";

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          meta = with pkgs.lib; {
            description = "A fast, memory-safe Rust library and CLI tool for reading, writing, and managing ASAR archives.";
            homepage = "https://github.com/vMohammad24/asar.rs/";
            license = licenses.agpl3Only;
            mainProgram = "asar";
          };
        };
      in {
        packages = {
          default = asar-cli;
          asar-cli = asar-cli;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = asar-cli;
          exePath = "/bin/asar";
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
            clippy
            rustfmt
            rust-analyzer
          ];
        };
      }
    );
}
