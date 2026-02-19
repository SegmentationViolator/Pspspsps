{
    inputs = {
        flake-utils.url = "github:numtide/flake-utils";
        naersk = {
            url = "github:nix-community/naersk";
            inputs.nixpkgs.follows = "nixpkgs";
        };
        nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    };

    outputs = { nixpkgs, flake-utils, naersk, ... }:
        flake-utils.lib.eachDefaultSystem (system:
            let
                pkgs = import nixpkgs { inherit system; };
                naersk' = pkgs.callPackage naersk { };
            in
            {
                packages.default = naersk'.buildPackage {
                    src = ./.;
                };

                devShell = pkgs.mkShell {
                    buildInputs = with pkgs; [ cargo jq perf rustc rustfmt rustPackages.clippy ];
                    RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
                };
            }
        );
}
