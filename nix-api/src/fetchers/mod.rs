use serde::{Deserialize, Serialize};

pub mod git;

// <https://github.com/NixOS/nix/blob/c9211b0b2d52a26ed666780b763b39a5bddd3fb3/src/libfetchers/include/nix/fetchers/fetchers.hh#L274>
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    pub r#type: String,
    pub key: String,
}
