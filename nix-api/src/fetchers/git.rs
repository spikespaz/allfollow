use serde::{Deserialize, Serialize};
use url::Url;

use super::PublicKey;

// <https://github.com/NixOS/nix/blob/c9211b0b2d52a26ed666780b763b39a5bddd3fb3/src/libfetchers/git.cc#L202-L219>
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitInputScheme {
    pub url: Url,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub shallow: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub submodules: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub lfs: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub export_ignore: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nar_hash: Option<String>,
    #[serde(default)]
    pub all_refs: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dirty_rev: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dirty_short_rev: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub verify_commit: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keytype: Option<String>,
    #[serde(default)]
    pub public_key: Option<PublicKey>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub public_keys: Vec<PublicKey>,
}

fn is_false(flag: &bool) -> bool {
    !flag
}
