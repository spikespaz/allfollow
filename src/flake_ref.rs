use serde::{Deserialize, Serialize};

/// All flake reference types according to:
/// <https://nix.dev/manual/nix/2.28/command-ref/new-cli/nix3-flake#types>
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename = "lowercase")]
pub enum Fetcher {
    Indirect,
    Path,
    Git,
    Mercurial,
    Tarball,
    File,
    GitHub,
    GitLab,
    SourceHut,
}
