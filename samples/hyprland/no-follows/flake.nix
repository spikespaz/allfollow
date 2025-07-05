{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    systems.url = "github:nix-systems/default-linux";

    aquamarine.url = "github:hyprwm/aquamarine";
    hyprcursor.url = "github:hyprwm/hyprcursor";
    hyprgraphics.url = "github:hyprwm/hyprgraphics";
    hyprland-protocols.url = "github:hyprwm/hyprland-protocols";
    hyprland-qtutils.url = "github:hyprwm/hyprland-qtutils";
    hyprlang.url = "github:hyprwm/hyprlang";
    hyprutils.url = "github:hyprwm/hyprutils";
    hyprwayland-scanner.url = "github:hyprwm/hyprwayland-scanner";
    xdph.url = "github:hyprwm/xdg-desktop-portal-hyprland";

    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = { ... }: { };
}
