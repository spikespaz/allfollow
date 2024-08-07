use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlakeLock {
    pub nodes: HashMap<String, InputNode>,
    pub root: String,
    pub version: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InputNode {
    #[serde(skip_serializing_if = "Clone::clone", default)]
    pub flake: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<HashMap<String, InputNodeRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<LockedInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<FlakeRef>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedInput {
    pub last_modified: usize,
    pub nar_hash: String,
    #[serde(flatten)]
    pub flake_ref: FlakeRef,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputNodeRef {
    Name(String),
    Follows(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum FlakeRef {
    Indirect {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        rev: Option<String>,
    },
    Tarball {
        url: String,
    },
    Git {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        r#ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        rev: Option<String>,
        #[serde(skip_serializing_if = "std::ops::Not::not", default)]
        submodules: bool,
    },
    Github {
        owner: String,
        repo: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        r#ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        rev: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        dir: Option<String>,
    },
}

impl FlakeLock {
    pub fn get_root_input(&self, name: impl AsRef<str>) -> Option<&InputNode> {
        let root_inputs = self.nodes.get(&self.root)?.inputs.as_ref()?;
        self.get_input_by_ref(root_inputs.get(name.as_ref())?)
    }

    pub fn get_input_by_ref(&self, r#ref: &InputNodeRef) -> Option<&InputNode> {
        match r#ref {
            InputNodeRef::Name(name) => self.nodes.get(name),
            InputNodeRef::Follows(path) => {
                let mut curr = self.get_root_input(path.first()?)?;
                for name in path.iter().skip(1) {
                    let r#ref = curr.inputs.as_ref()?.get(name)?;
                    curr = self.get_input_by_ref(r#ref)?;
                }
                Some(curr)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FlakeLock;

    macro_rules! parse_lock_file {
        ($test_ident:ident, $lock_file_path:literal) => {
            #[test]
            fn $test_ident() {
                static SOURCE: &str = include_str!($lock_file_path);
                let deser = &mut serde_json::Deserializer::from_str(SOURCE);
                let res: Result<FlakeLock, _> = serde_path_to_error::deserialize(deser);
                match res {
                    Ok(lock) => {
                        dbg!(lock);
                    }
                    Err(e) => panic!("{}", e),
                }
            }
        };
    }

    parse_lock_file!(parse_hyprnix_before, "../samples/hyprnix/before/flake.lock");
    parse_lock_file!(parse_hyprnix_after, "../samples/hyprnix/after/flake.lock");
}
