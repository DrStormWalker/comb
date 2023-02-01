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
  flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          ( import rust-overlay )
        ];
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default;

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      comb = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;

        cargoExtraArgs = "--target thumbv6m-none-eabi";

        doCheck = false;

        buildInputs = with pkgs; [

        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
        ];
      };
    in {
      checks = {
        inherit comb;
      };

      packages.default = comb;

      devShells.default = pkgs.buildEnv {
        inputsFrom = builtins.attrValues self.checks;

        nativeBuildInputs = with pkgs; [
          rustToolchain

          clippy
          rustfmt
          rust-analyzer
        ];
      };
    }
  );
}
