{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        inherit (nixpkgs) lib;

        pkgs = nixpkgs.legacyPackages.${system};
        rpath = lib.makeLibraryPath (with pkgs; [
          fontconfig
          libxkbcommon
          wayland
        ]);
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "yofi";
          inherit ((lib.importTOML (self + "/Cargo.toml")).package) version;

          src = self;

          cargoLock.lockFile = self + "/Cargo.lock";

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            libxkbcommon
          ];

          postFixup = ''
            patchelf $out/bin/yofi --add-rpath ${rpath}
          '';
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cargo
            clippy
            libxkbcommon
            pkg-config
            rust-analyzer
            rustc
            rustfmt
          ];

          LD_LIBRARY_PATH = rpath;
        };
      }
    );
}
