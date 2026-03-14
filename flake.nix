{
    description = "A lambda calculus evaluator";

    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
        crane.url = "github:ipetkov/crane";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs =
        {
            self,
            nixpkgs,
            crane,
            flake-utils,
            ...
        }:
        flake-utils.lib.eachDefaultSystem (
            system:
            let
                pkgs = import nixpkgs { inherit system; };
                mingwPkgs = import nixpkgs {
                    inherit system;
                    crossSystem = {
                        config = "x86_64-w64-mingw32";
                    };
                };

                craneLib = crane.mkLib pkgs;
                mingwCraneLib = crane.mkLib mingwPkgs;

                src = craneLib.cleanCargoSource ./.;

                commonArgs = {
                    inherit src;
                    strictDeps = true;
                };

                cargoArtifacts = craneLib.buildDepsOnly commonArgs;
                mingwCargoArtifacts = mingwCraneLib.buildDepsOnly (
                    commonArgs
                    // {
                        CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
                    }
                );

                default = craneLib.buildPackage (
                    commonArgs
                    // {
                        inherit cargoArtifacts;
                    }
                );

                windows = mingwCraneLib.buildPackage (
                    commonArgs
                    // {
                        cargoArtifacts = mingwCargoArtifacts;
                        CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
                        CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER =
                            "${mingwPkgs.stdenv.cc.targetPrefix}gcc";
                    }
                );
            in
            {
                checks = {
                    crate-clippy = craneLib.cargoClippy (
                        commonArgs
                        // {
                            inherit cargoArtifacts;
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
