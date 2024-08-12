# Allfollow

# What is this?

Take this flake's inputs, for example.

```nix
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
  };
```

This would be the content of the `flake.lock` for the above inputs,

```json
{
  "nodes": {
    "nixpkgs": {
      // ...
    },
    "nixpkgs_2": {
      // ...
    },
    "rust-overlay": {
      "inputs": {
        "nixpkgs": "nixpkgs_2"
      },
      // ...
    },
    "systems": {
      // ...
    },
    "root": {
      "inputs": {
        "nixpkgs": "nixpkgs",
        "rust-overlay": "rust-overlay",
        "systems": "systems"
      }
    }
  },
  "root": "root",
  "version": 7
}
```

You can see two instances of `nixpkgs`. This means our flake's closure size
is rather large, and we don't really want to download `nixpkgs` twice.

This can be fixed by defining `rust-overlay.inputs.nixpkgs.follows` to the name
of our `nixpkgs`.

```nix
  inputs = {
    # ...
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # ...
  };
```

That change would result in the following in the lock file:

```diff
  {
    "nodes": {
      "nixpkgs": {
        // ...
      },
-     "nixpkgs_2": {
-       // ...
-     },
      "rust-overlay": {
        "inputs": {
-         "nixpkgs": "nixpkgs_2"
+         "nixpkgs": [
+           "nixpkgs"
+         ]
        },
        // ...
      },
      // ...
    },
    "root": "root",
    "version": 7
  }
```

This tool, `allfollow`, will effectively do the same as a post-process to your
`flake.lock`.

# Okay, why?

Large flakes that aggregate other packages need to add
`inputs.*.inputs.*.follows = "*"` for each input of every top-level input,
perhaps even recursively.

Take a look at the [Hyprland's `flake.nix`]. Every single repository in the
HyprWM organization with a `flake.nix` is riddled with the words `inputs`
and `follows`, and it is a pain to maintain this web of dependencies manually.

As of writing, for my flake [Hyprnix], I was successfully able to remove
47+ tediously-maintained lines from the `flake.nix`. the original `flake.lock`
is 933 lines long, the only 150 after using `allfollow`.

[Hyprnix]: https://github.com/hyprland-community/hyprnix
