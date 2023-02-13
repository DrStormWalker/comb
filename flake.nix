{
  description = "CoMB (Corroded Macro Bindings). A program to map gamepad inputs to keyboard keys";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
    ...
  }:
  flake-utils.lib.eachSystem ["aarch64-linux" "x86_64-linux"] (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          ( import rust-overlay )
        ];
      };

      rustToolchain = pkgs.rust-bin.nightly.latest.default;

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
    in {
      overlays.default = _: prev: {
        comb = prev.callPackage ./nix/default.nix {
          inherit craneLib;
        };
      };

      packages = (self.overlays.${system}.default null pkgs)
        // {
          default = self.packages.${system}.comb;
        };
    
      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.checks;

        nativeBuildInputs = with pkgs; [
          rustToolchain

          rust-analyzer
          clippy
          rustfmt
          cargo-expand
        ];
      };
    }
  );
}
