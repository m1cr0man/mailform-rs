{
  description = "Mailform - Contact us form processor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      # inputs.rust-analyzer-src.follows = "";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        envVars = {
          OPENSSL_NO_VENDOR = "1";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          PKG_CONFIG = "${pkgs.pkg-config}/bin/pkg-config";
        };

        stdenv =
          if pkgs.stdenv.isLinux then
            pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv
          else
            pkgs.stdenv;

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        mkToolchain = fenix.packages.${system}.combine;

        toolchain = fenix.packages.${system}.stable;

        buildToolchain = mkToolchain (with toolchain; [
          cargo
          rustc
        ]);

        craneLibBuild = craneLib.overrideToolchain buildToolchain;

        devToolchain = mkToolchain (with toolchain; [
          cargo
          clippy
          rust-src
          rustc
          llvm-tools
          rust-analyzer

          # Always use nightly rustfmt because most of its options are unstable
          fenix.packages.${system}.latest.rustfmt
        ]);

        craneLibDev = craneLib.overrideToolchain devToolchain;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src stdenv;
          strictDeps = true;

          buildInputs = [
            # Add additional build inputs here
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
        } // envVars;

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLibBuild.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        mailform-rs = craneLibBuild.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          cargoExtraArgs = "--locked -F cli";
        });
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit mailform-rs;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          mailform-rs-clippy = craneLibDev.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          mailform-rs-doc = craneLibDev.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });

          # Check formatting
          mailform-rs-fmt = craneLibDev.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          mailform-rs-audit = craneLibDev.cargoAudit {
            inherit src advisory-db;
          };

          # Audit licenses
          mailform-rs-deny = craneLibDev.cargoDeny {
            inherit src;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `mailform-rs` if you do not want
          # the tests to run twice
          mailform-rs-nextest = craneLibDev.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });

          overlay = (import nixpkgs {
            inherit system;
            overlays = [ self.overlays.${system}.mailform-rs-nixpkgs ];
          }).mailform-rs;
        };

        packages = {
          inherit mailform-rs;
          default = mailform-rs;
          mailform-rs-lib = craneLibBuild.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
          mailform-rs-llvm-coverage = craneLibDev.cargoLlvmCov (commonArgs // {
            inherit cargoArtifacts;
          });
          devTools = pkgs.linkFarm "vscode-dev-tools" {
            inherit (pkgs) nixpkgs-fmt rnix-lsp gcc pkg-config;
            openssl = pkgs.openssl.dev;
            rust = devToolchain;
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = mailform-rs;
        };

        overlays = {
          mailform-rs-nixpkgs =
            let
              cargoConfig = (builtins.fromTOML (builtins.readFile "${self}/Cargo.toml"));
              pname = cargoConfig.package.name;
            in
            final: prev: {
              ${pname} = final.rustPlatform.buildRustPackage
                (envVars // rec {
                  inherit pname;
                  version = cargoConfig.package.version;

                  src = self;

                  cargoHash = "sha256-j3B41omVog8J4yhuaYnolzm/F+QwGHTPyrgUjbvW8IE=";

                  meta = with final.lib; {
                    description = "Contact us form processor";
                    homepage = "https://github.com/m1cr0man/mailform-rs";
                    license = licenses.asl20;
                    maintainers = [ maintainers.m1cr0man ];
                  };
                });
            };
        };

        devShells.default = craneLibDev.devShell
          {
            # Inherit inputs from checks.
            checks = self.checks.${system};

            # Additional dev-shell environment variables can be set directly
            # MY_CUSTOM_DEVELOPMENT_VAR = "something else";
            RUST_SRC_PATH = "${devToolchain}/lib/rustlib/src/rust/library";

            # Extra inputs can be added here; cargo and rustc are provided by default.
            packages = [
              # pkgs.ripgrep
            ];
          } // envVars;
      }
    );
}
