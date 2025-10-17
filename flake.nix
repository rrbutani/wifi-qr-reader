{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  nixConfig.extra-substituters = [ "https://cache.garnix.io" ];
  nixConfig.extra-trusted-public-keys = [ "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=" ];

  outputs = { self, nixpkgs, flake-utils }: let
    manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    pname = manifest.package.name;

    package =
      { lib, rustPlatform }: rustPlatform.buildRustPackage {
        inherit pname;
        inherit (manifest.package) version;

        src = with lib.fileset; toSource {
          root = ./.;
          fileset = unions [ ./src ./Cargo.toml ./Cargo.lock ];
        };
        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [ rustPlatform.bindgenHook ];

        meta = {
          homepage = "https://github.com/ComputerDruid/wifi-qr-reader";
          maintainers = with lib.maintainers; [ computerdruid ]; # :P
          licenses = with lib.licenses; [ mit asl20 ];
          platforms = lib.platforms.all;
        };
      }
    ;
  in {
    overlays.default = final: prev: {
      ${pname} = final.callPackage package {};
      lib = prev.lib.extend (_: prev_lib: {
        maintainers = prev_lib.maintainers // {
          computerdruid = {
            email = "ComputerDruid@gmail.com";
            github = "ComputerDruid";
            githubId = 34696;
            name = "Dan Johnson";
          };
        };
      });
    };
  } // flake-utils.lib.eachDefaultSystem (system: let
    pkgs = nixpkgs.legacyPackages.${system}.extend self.overlays.default;
  in rec {
    packages.default = pkgs.${pname};
    devShells.default = with pkgs; mkShell {
      inputsFrom = [ packages.default ];
      packages = [ rust-analyzer rustfmt clippy ];
    };
  });
}
