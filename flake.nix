{
  description = "htpc-cec-ctrl";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    {
      self,
      nixpkgs,
      systems,
      flake-utils,
    }:
    flake-utils.lib.eachSystem (import systems) (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages = rec {
          htpc-cec-ctrl = pkgs.callPackage ./nix/pkg.nix { };
          default = htpc-cec-ctrl;
        };

        devShells = {
          default = pkgs.callPackage ./nix/shell.nix { };
        };
      }
    )
    // {
      nixosModules.default =
        { ... }:
        {
          # Allow normal user to set user slice CPUQuota
          # Allow normal user to shut down
          security.polkit.extraConfig = ''
            polkit.addRule(function(action, subject) {
              if (
                action.id == "org.freedesktop.systemd1.manage-unit-files" &&
                action.lookup("verb") == "set-property" &&
                action.lookup("unit") == "user-1000.slice"
              ) {
                return polkit.Result.YES;
              }
            });
            polkit.addRule(function(action, subject) {
              if (action.id == "org.freedesktop.login1.power-off") {
                return polkit.Result.YES;
              }
            });
          '';
        };
      homeModules.default =
        { pkgs, ... }:
        {
          systemd.user.services.htpc-cec-ctrl = {
            Unit = {
              Description = "CEC controller";
              After = [ "network.target" ];
            };

            Install = {
              WantedBy = [ "default.target" ];
            };

            Service = {
              ExecStartPre = "${self.packages.htpc-cec-ctrl}/bin/htpc-cec-ctrl unrestrict-cpu";
              ExecStart = "${self.packages.htpc-cec-ctrl}/bin/htpc-cec-ctrl";
              ExecStopPost = "${self.packages.htpc-cec-ctrl}/bin/htpc-cec-ctrl unrestrict-cpu";
              Restart = "always";
              RestartSec = 30;
              Environment = "RUST_LOG=htpc_cec_ctrl=info";
            };
          };
        };
    };
}
