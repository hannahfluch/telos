{
  inputs = {
    systems.url = "github:nix-systems/default";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.systems.follows = "systems";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      flake-utils,
      nixpkgs,
      naersk,
      fenix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) { inherit system; };
        lib = pkgs.lib;
        fenixPkgs = fenix.packages.${system};
        rustSrc = fenixPkgs.latest.rust-src;
        rustToolchain = fenixPkgs.combine [
          fenixPkgs.latest.cargo
          fenixPkgs.latest.rustc
          rustSrc
          fenixPkgs.targets.x86_64-unknown-uefi.latest.rust-std
          fenixPkgs.targets.x86_64-unknown-linux-gnu.latest.rust-std
        ];

        naersk' = pkgs.callPackage naersk {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        buildPackage = lib.makeOverridable naersk'.buildPackage;

        commonArgs = pname: {
          inherit pname;
          cargoBuildOptions =
            x:
            x
            ++ [
              "-p"
              "${pname}"
            ];
          release = true;

          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./.cargo
              ./Cargo.toml
              ./Cargo.lock
              ./kernel
              ./loader
            ];
          };

          strictDeps = true;
          doCheck = false; # can't find crate for `test`
          buildInputs = [ ];
          additionalCargoLock = "${rustSrc}/lib/rustlib/src/rust/library/Cargo.lock"; # for building std
        };
        kernel =
          let
            base = commonArgs "kernel";
          in
          buildPackage (
            base
            // {
              CARGO_BUILD_TARGET = "${./kernel/x86_64-unknown-telos.json}";
              dontStrip = true; # breaks kernel
              cargoBuildOptions = x: base.cargoBuildOptions x ++ [ "-Zjson-target-spec" ];
            }
          );
        loader = buildPackage (
          (commonArgs "loader")
          // {
            CARGO_BUILD_TARGET = "x86_64-unknown-uefi";
          }
        );
        bootimage = pkgs.callPackage ./nix/img.nix { inherit kernel loader; };
        qemu = pkgs.callPackage ./nix/qemu.nix { inherit bootimage; };
        flash = pkgs.callPackage ./nix/flash.nix { inherit bootimage; };

      in
      {

        packages = {
          default = qemu;
          inherit
            kernel
            loader
            qemu
            flash
            bootimage
            ;
        };
        # For `nix develop`:
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
          ];
        };

      }
    );

}
