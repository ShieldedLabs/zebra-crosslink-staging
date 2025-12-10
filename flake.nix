# Build, testing, and developments specification for the `nix` environment
#
# # Prerequisites
#
# - Install the `nix` package manager: https://nixos.org/download/
# - Configure `flake` support: https://nixos.wiki/wiki/Flakes
#
# # Build
#
# ```
# $ nix build --print-build-logs
# ```
#
# This produces:
#
# - ./result/bin/zebra-scanner
# - ./result/bin/zebrad-for-scanner
# - ./result/bin/zebrad
# - ./result/book/
#
# The book directory is the root of the book source, so to view the rendered book:
#
# ```
# $ xdg-open ./result/book/book/index.html
# ```
#
# # Development
#
# ```
# $ nix develop
# ```
#
# This starts a new subshell with a development environment, such as
# `cargo`, `clang`, `protoc`, etc... So `cargo test` for example should
# work.
{
  description = "The zebra zcash node binaries and crates with Crosslink protocol features";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # TODO: Switch to `flake-parts` lib for cleaner organization.
    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        project-name = "zebra-crosslink";

        # Local utility library:
        flakelib = import ./flake inputs {
          pname = "${project-name}-workspace";
          src-root = ./.;
          rust-toolchain-toml = ./rust-toolchain.toml;
          inherit system;
        };

        inherit (flakelib)
          build-rust-workspace
          crane-dev-shell
          links-table
          nixpkgs
          run-command
          select-source
          ;

        # We use this style of nix formatting in checks and the dev shell:
        nixfmt = nixpkgs.nixfmt-rfc-style;

        # We use the latest nixpkgs `libclang`:
        inherit (nixpkgs.llvmPackages) libclang;

        src-book = select-source {
          name-suffix = "book";
          paths = [
            ./book
            ./README.md
          ];
        };

        src-rust = select-source {
          name-suffix = "rust";
          paths = [
            ./.cargo
            ./.config
            ./Cargo.lock
            ./Cargo.toml
            ./clippy.toml
            ./crosslink-test-data
            ./release.toml
            ./rust-toolchain.toml
            ./tower-batch-control
            ./tower-fallback
            ./zebra-chain
            ./zebra-consensus
            ./zebra-crosslink
            ./zebra-grpc
            ./zebra-network
            ./zebra-node-services
            ./zebra-rpc
            ./zebra-scan
            ./zebra-script
            ./zebra-state
            ./zebra-test
            ./zebra-utils
            ./zebrad
          ];
        };

        zebrad-outputs = build-rust-workspace ./zebrad {
          src = src-rust;

          strictDeps = true;

          # Note: we disable tests since we'll run them all via cargo-nextest
          doCheck = false;

          # Use the clang stdenv, overriding any downstream attempt to alter it:
          stdenv = _: nixpkgs.llvmPackages.stdenv;

          nativeBuildInputs = with nixpkgs; [
            pkg-config
            protobuf
          ];

          buildInputs = with nixpkgs; [
            libclang
            rocksdb
          ];

          # Additional environment variables can be set directly
          LIBCLANG_PATH = "${libclang.lib}/lib";
        };

        zebrad = zebrad-outputs.pkg;

        zebra-book = nixpkgs.stdenv.mkDerivation rec {
          name = "zebra-book";
          src = src-book;
          buildInputs = with nixpkgs; [
            graphviz
            mdbook
            mdbook-admonish
            mdbook-graphviz
            mdbook-katex
            mdbook-linkcheck
            mdbook-mermaid
          ];
          builder = nixpkgs.writeShellScript "${name}-builder.sh" ''
            if mdbook build --dest-dir "$out/book/book" "$src/book" 2>&1 | grep -E 'ERROR|WARN'
            then
              echo 'Failing due to mdbook errors/warnings.'
              exit 1
            fi
          '';
        };
      in
      {
        packages = (
          let
            base-pkgs = {
              inherit
                zebrad
                zebra-book
                src-book
                src-rust
                ;
            };

            all = links-table "all" {
              "./bin" = "${zebrad}/bin";
              "./book" = "${zebra-book}/book";
              "./src/${project-name}/book" = "${src-book}/book";
              "./src/${project-name}/rust" = src-rust;
            };
          in

          base-pkgs
          // {
            inherit all;
            default = all;
          }
        );

        checks = (
          zebrad-outputs.checks
          // {
            # Build the crates as part of `nix flake check` for convenience
            inherit zebrad;

            # Check formatting
            nixfmt-check = run-command "nixfmt" [ nixfmt ] ''
              set -efuo pipefail
              exitcode=0
              for f in $(find '${./.}' -type f -name '*.nix')
              do
                cmd="nixfmt --check --strict \"$f\""
                echo "+ $cmd"
                eval "$cmd" || exitcode=1
              done
              [ "$exitcode" -eq 0 ] && touch "$out" # signal success to nix
              exit "$exitcode"
            '';
          }
        );

        apps = {
          zebrad = inputs.flake-utils.lib.mkApp { drv = zebrad; };
        };

        # TODO: BEWARE: This dev shell may have buggy deviations from the build.
        devShells.default = crane-dev-shell {
          inputsFrom = [
            zebrad
            zebra-book
          ];

          packages = with nixpkgs; [ cargo-nextest ];

          LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath (
            with nixpkgs;
            [
              libGL
              libxkbcommon
              xorg.libX11
              xorg.libxcb
              xorg.libXi
            ]
          );
        };
      }
    );
}
