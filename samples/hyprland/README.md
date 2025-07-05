## The `hyprland` sample is based on the [Hyprland] flake.

Each directory in this tree contains a `flake.nix` and corresponding `flake.lock`.

- [`with-follows/`](./with-follows) - Matches upstream as of [3c9447ca53f76abd1372bca5749c9ef701fb76c0].
  - At the time of that commit, Hyprland is still using manual `follows` declarations for every transitive input.
  - [`flake.nix`](./with-follows/flake.nix) - Dummy version of [Hyprland's `flake.nix`] with everything removed except for the `inputs` attributes, which is kept verbatim.
  - [`flake.lock`](./with-follows/flake.lock) - Verbatim copy of [Hyprland's `flake.lock`].
    - This serves as a control sample to compare against when verifying Allfollow's behavior, which should produce functionally equivalent output.

[Hyprland]: https://github.com/hyprwm/Hyprland
[3c9447ca53f76abd1372bca5749c9ef701fb76c0]: https://github.com/hyprwm/Hyprland/commit/3c9447ca53f76abd1372bca5749c9ef701fb76c0
[Hyprland's `flake.nix`]: https://github.com/hyprwm/Hyprland/blob/3c9447ca53f76abd1372bca5749c9ef701fb76c0/flake.nix
[Hyprland's `flake.lock`]: https://github.com/hyprwm/Hyprland/blob/3c9447ca53f76abd1372bca5749c9ef701fb76c0/flake.lock
