{
  description = "Nixette comfort-zone profile";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixette-overlays.url = "github:nixette/overlays";
    nixette-style.url = "github:nixette/style-pack";
  };

  outputs = { self, nixpkgs, nixette-overlays, nixette-style, ... }@inputs:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ nixette-overlays.overlays.nix-emerge ];
      };
    in
    {
      nixosConfigurations.nixette-comfort-zone = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          ./profiles/comfort-zone.nix
          ({ config, pkgs, ... }:
            {
              nixpkgs.config.allowUnfree = true;
              environment.systemPackages = with pkgs; [
                nixette-style
                steam
                lutris
                krita
              ];

              services.nixette.nix-emerge = {
                enable = true;
                ebuilds = [
                  "games-emulation/gamescope"
                  "media-sound/pipewire"
                ];
              };

              services.nixette.affirmd.enable = true;
              services.nixette.affirmd.pronouns = "she/her";
              services.nixette.affirmd.motdPath = ./affirmations.yml;

              programs.plasma.enable = true;
              services.displayManager.sddm.enable = true;
              services.displayManager.sddm.theme = nixette-style.themes.catgirl-sunrise;

              users.users.nixie = {
                isNormalUser = true;
                extraGroups = [ "wheel" "audio" "input" "video" ];
                shell = pkgs.zsh;
              };

              programs.zsh.promptInit = ''
                eval "$(nixette-style prompt --name nixie --pronouns she/her)"
              '';
            })
        ];
      };
    };
}
