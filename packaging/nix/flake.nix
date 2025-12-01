{
  description = "Palemachine - Agentic AI coding assistant backend";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        
        manifest = (pkgs.lib.importTOML ../../Cargo.toml).package;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;
          
          src = ../../.;

          cargoLock = {
            lockFile = ../../Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper ];
          
          buildInputs = [ 
            pkgs.openssl 
            pkgs.sqlite 
            pkgs.python3
          ];

          # Disable tests that require network or local DB if necessary
          doCheck = false;

          postInstall = ''
            mkdir -p $out/share/palemachine
            cp -r pages $out/share/palemachine/
            cp downloader $out/share/palemachine/
            cp requirement.txt $out/share/palemachine/
            cp config.toml.example $out/share/palemachine/
            cp .version $out/share/palemachine/
            cp bambam_morigatsu_chuapo.sh $out/share/palemachine/
            cp update.sh $out/share/palemachine/
            
            wrapProgram $out/bin/palemachine \
              --chdir $out/share/palemachine
          '';

          meta = with pkgs.lib; {
            description = manifest.description or "Palemachine backend";
            homepage = "https://github.com/Xmoncoco/palemachine";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
            pkg-config
            openssl
            sqlite
            ffmpeg
            python3
            python3Packages.virtualenv
          ];
          
          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.openssl.out}/lib:$LD_LIBRARY_PATH
          '';
        };
      }
    );
}
