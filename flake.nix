{
  description = "Turxo";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      fenix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit overlays system; };
        rustToolchain = pkgs.fenix.stable;

        rustPkg = rustToolchain.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
        ];

        optionals =
          with pkgs;
          lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

        nixfmt = nixpkgs.legacyPackages.${system}.nixfmt-rfc-style;

        erlang = pkgs.beam.packagesWith pkgs.beam.interpreters.erlang_28;
        packages = [
          erlang.elixir
          rustPkg
          nixfmt
        ] ++ optionals;

      in
      {
        devShells = {
          default = pkgs.mkShellNoCC {
            buildInputs = packages;
            shellHook = ''
              # this allows mix to work on the local directory
              mkdir -p .nix-mix .nix-hex
              export MIX_HOME=$PWD/.nix-mix
              export HEX_HOME=$PWD/.nix-mix

              # make hex from Nixpkgs available
              # `mix local.hex` will install hex into MIX_HOME and should take precedence
              export MIX_PATH="${erlang.hex}/lib/erlang/lib/hex/ebin"
              export PATH=$MIX_HOME/bin:$HEX_HOME/bin:$PATH

              # keep your shell history in iex
              export ERL_AFLAGS="-kernel shell_history enabled"         
            '';
          };
        };
        formatter = nixfmt;
      }
    );
}
