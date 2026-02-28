{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            erlang
            rebar3

            rustc
            cargo
          ];

          buildInputs = with pkgs; [
            erlang-language-platform

            rustfmt
            rust-analyzer
            clippy
          ];

          shellHook = ''
            mkdir -p .erlang
            export MIX_HOME=$PWD/.erlang/mix
            export HEX_HOME=$PWD/.erlang/hex
            export ERL_LIBS=$HEX_HOME/lib/erlang/lib
          '';
        };
      }
    );
}
