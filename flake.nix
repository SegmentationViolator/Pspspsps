{
    description = "A lambda calculus evaluator";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
        crane.url = "github:ipetkov/crane";
        flake-utils.url = "github:numtide/flake-utils";
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };

    outputs =
        {
            self,
            nixpkgs,
            crane,
            flake-utils,
            rust-overlay,
            ...
        }:
        flake-utils.lib.eachDefaultSystem (
            system:
            let
                windowsTarget = "x86_64-pc-windows-gnu";

                pkgs = import nixpkgs {
                    inherit system;
                    overlays = [ (import rust-overlay) ];
                };

                mingwPkgs = import nixpkgs {
                    localSystem = system;
                    crossSystem = {
                        config = "x86_64-w64-mingw32";
                        libc = "ucrt";
                    };
                    overlays = [ (import rust-overlay) ];
                };

                rustToolchain = pkgs.rust-bin.stable.latest.default.override {
                    targets = [ windowsTarget ];
                };

                craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);

                src = craneLib.cleanCargoSource ./.;

                commonArgs = {
                    inherit src;
                    strictDeps = true;
                };

                cargoArtifacts = craneLib.buildDepsOnly commonArgs;
                windowsArgs = {
                    CARGO_BUILD_TARGET = windowsTarget;
                    CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER =
                        "${mingwPkgs.stdenv.cc}/bin/${mingwPkgs.stdenv.cc.targetPrefix}cc";
                    RUSTFLAGS = "-L native=${mingwPkgs.windows.pthreads}/lib";
                    nativeBuildInputs = [
                        mingwPkgs.stdenv.cc
                        mingwPkgs.stdenv.cc.bintools
                    ];
                };

                windowsCargoArtifacts = craneLib.buildDepsOnly (
                    commonArgs
                    // windowsArgs
                );

                default = craneLib.buildPackage (
                    commonArgs
                    // {
                        inherit cargoArtifacts;
                    }
                );

                windows = craneLib.buildPackage (
                    commonArgs
                    // windowsArgs
                    // {
                        cargoArtifacts = windowsCargoArtifacts;
                        doCheck = false;
                    }
                );
            in
            {
                checks = {
                    crate-clippy = craneLib.cargoClippy (
                        commonArgs
                        // {
                            inherit cargoArtifacts;
                            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
                        }
                    );
                };

                packages = {
                    inherit default;
                    inherit windows;
                };

                devShells.default = craneLib.devShell {
                    checks = self.checks.${system};
                    packages = [];
                };
            }
        );
}
