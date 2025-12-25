{
  description = "moss agent interconnect layer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            stdenv.cc.cc
            ripgrep
            jq
            sqlite
            # VS Code extension development
            nodejs_22
            nodePackages.npm
            nodePackages.typescript
            # Rust toolchain
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt
          ];
	  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH";
        };
      }
    );
}
