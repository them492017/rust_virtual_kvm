{
  description = "A software kvm switch written in rust";

  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.75.0";
          packageFun = import ./Cargo.nix;
          packageOverrides = pkgs: pkgs.rustBuilder.overrides.all;
        };

        rustVirtualKvm = (rustPkgs.workspace.rust_virtual_kvm {});

        workspaceShell = (rustPkgs.workspaceShell {
            # packages = [ cargo2nix ];
            # nativeBuildInputs = cargo2nix.nativeBuildInputs;
        });

      in rec {
        # packages = {
        #   # rust_virtual_kvm = (rustPkgs.workspace.rust_virtual_kvm {});
        #   server = (rustPkgs.workspace.server {});
        #   client = (rustPkgs.workspace.client {});
        #   default = packages.server;
        # };
        packages = {
          client = pkgs.runCommand "client" {
            buildInputs = [ rustVirtualKvm ];
          } ''
            mkdir -p $out/bin
            cp ${rustVirtualKvm}/bin/client $out/bin/client
          '';
          server = pkgs.runCommand "server" {
            buildInputs = [ rustVirtualKvm ];
          } ''
            mkdir -p $out/bin
            cp ${rustVirtualKvm}/bin/server $out/bin/server
          '';
          default = packages.client;
        };

        devShells = {
          default = workspaceShell;
        };

        workspaceAttrs = builtins.attrNames rustPkgs.workspace;
      }
    );
}
