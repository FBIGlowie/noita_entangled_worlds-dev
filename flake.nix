{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
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
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
        latest = {
	ver = "master";
	sha256 = "sha256-Bdk60+FJHvpw8J0s7LoifEGZCImRuupwzu/7/N8jbCs=";
        };
        mkProxy =
          { ver, sha256 }:
          pkgs.stdenv.mkDerivation rec {
            name = "noita-proxy-launcher";
            src = pkgs.fetchurl {
              url = "https://github.com/IntQuant/noita_entangled_worlds/releases/download/${ver}/noita-proxy-linux.zip";
              inherit sha256;
            };
            nativeBuildInputs = [ pkgs.unzip ];
            buildInputs = [
              pkgs.steam-run
            ];
            unpackPhase = ''
              unzip $src
            '';
            installPhase = ''
              mkdir -p $out/bin
              cp -r * $out/bin
              chmod +x $out/bin/*
            '';
          };

      in
      rec {
        latest-noita-proxy = mkProxy {
          ver = latest.ver;
          sha256 = latest.sha256;
        };
        packages = rec {
          default = pkgs.writeShellScriptBin "noita-entangled" ''
            steam-run ${latest-noita-proxy}/bin/noita_proxy.x86_64
          '';
        };
      }
    );
}
