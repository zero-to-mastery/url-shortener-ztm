{
  description = "Rust devShell with Fenix stable toolchain";

  # Nix configuration: allow binary cache from nix-community to speed up builds
  nixConfig = {
    extra-substituters = [ "https://nix-community.cachix.org" ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs"
    ];
  };

  inputs = {
    # Nixpkgs (unstable) for access to up-to-date packages
    nixpkgs.url = "nixpkgs/nixos-unstable";

    # Flake utils: helps iterate over multiple systems easily (e.g., x86_64-linux, aarch64-linux, macos)
    flake-utils.url = "github:numtide/flake-utils";

    # Rust-specific tools for analysis lifetime and ownership
    rustowl-flake.url = "github:nix-community/rustowl-flake";

    # Pre-commit hooks for Nix projects
    git-hooks.url = "github:cachix/git-hooks.nix";

    # Fenix: Rust toolchain via Nix (stable, nightly, minimal)
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-utils,
      fenix,
      rustowl-flake,
      git-hooks,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Use LLVM 21 for clang/lld integration with Rust builds
        llvm = pkgs.llvmPackages_21;

        # Select the stable Rust toolchain from Fenix with required components
        fenixPkgs = fenix.packages.${system}.stable;
        rustToolchain = fenixPkgs.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
          "rust-analyzer"
        ];

        # Define git pre-commit hooks configuration
        pre-commit-check = git-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            # Enable Nix formatting check (nixfmt)
            nixfmt-rfc-style.enable = true;

            # Rust formatting (disabled by default)
            # rustfmt.enable = true;

            # Clippy linting (enable by default)
            clippy = {
              enable = true;
              packageOverrides = {
                cargo = fenixPkgs.cargo;
                clippy = fenixPkgs.clippy;
              };
              settings = {
                allFeatures = true;
                denyWarnings = true;
                extraArgs = "--all-targets";
              };
            };
          };
        };

        # Common tools needed for building and development
        commonBuildInputs = with pkgs; [
          rustowl-flake.packages.${system}.rustowl
          rustToolchain
          llvm.clang
          llvm.lldb # debug with lldb-dap in VSCode
          llvm.lld
          llvm.libllvm
          cargo-llvm-cov

          openssl
          sqlx-cli
          sqlite
        ];

        # Convert ymal to JSON for environment preparation
        baseConfig = pkgs.runCommand "yaml-to-json" { } ''
          ${pkgs.yq-go}/bin/yq -o=json '.' ${./configuration/base.yml} > $out
        '';
        finalConfig = builtins.fromJSON (builtins.readFile baseConfig);

        # Rust ≥1.90 uses lld by default. On NixOS this can fail due to missing system linker.
        # We set clang + lld explicitly to avoid "linker not found" errors.
        commonRustEnv = {
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${llvm.clang}/bin/clang";
          RUSTFLAGS = "-Clink-arg=-fuse-ld=lld";
          LIBCLANG_PATH = "${llvm.libclang.lib}/lib";
        };
      in
      {
        # Use nixfmt-rfc-style as default formatter for `nix fmt`
        formatter = pkgs.nixfmt-rfc-style;

        devShells = {
          default = pkgs.mkShell (
            commonRustEnv
            // {
              buildInputs = commonBuildInputs;
              nativeBuildInputs = [ pkgs.pkg-config ];

              shellHook = ''
                # Export key variables from the parsed config
                # export API_KEY=${finalConfig.application.api_key}

                # Ensure OpenSSL libs are available at runtime
                export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.openssl ]}:$LD_LIBRARY_PATH"

                # enable pre-commit hook installation inside the dev shell
                ${pre-commit-check.shellHook}

                echo "[info] Using Fenix (stable) Rust toolchain."
              '';
            }
          );
        };

        # Optional: expose pre-commit checks to CI (e.g. `nix flake check`)
        # checks = { inherit pre-commit-check; };
      }
    );
}
