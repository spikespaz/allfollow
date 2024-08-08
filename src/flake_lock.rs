use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const SUPPORTED_LOCK_VERSION: u32 = 7;

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
    pub fn new() -> Self {
        static ROOT: &str = "root";
        Self {
            nodes: HashMap::from_iter([(
                ROOT.to_owned(),
                InputNode {
                    flake: true,
                    inputs: Some(HashMap::new()),
                    ..Default::default()
                },
            )]),
            root: ROOT.to_owned(),
            version: SUPPORTED_LOCK_VERSION,
        }
    }

    pub fn root(&self) -> &InputNode {
        self.nodes
            .get(&self.root)
            .expect("the root node to already exist")
    }

    fn root_mut(&mut self) -> &mut InputNode {
        self.nodes
            .get_mut(&self.root)
            .expect("the root node to already exist")
    }

    pub fn input_refs(&self) -> &HashMap<String, InputNodeRef> {
        self.root()
            .inputs
            .as_ref()
            .expect("the root node to have been initialized")
    }

    fn input_refs_mut(&mut self) -> &mut HashMap<String, InputNodeRef> {
        self.root_mut()
            .inputs
            .as_mut()
            .expect("the root node to have been initialized")
    }

    pub fn get_input_by_name(&self, name: impl AsRef<str>) -> Option<&InputNode> {
        self.input_refs()
            .get(name.as_ref())
            .and_then(|r#ref| self.get_node_by_ref(r#ref))
    }

    pub fn get_node_by_ref(&self, r#ref: &InputNodeRef) -> Option<&InputNode> {
        match r#ref {
            InputNodeRef::Name(name) => self.nodes.get(name),
            InputNodeRef::Follows(path) => {
                let mut curr = self.get_input_by_name(path.first()?)?;
                for name in path.iter().skip(1) {
                    let r#ref = curr.inputs.as_ref()?.get(name)?;
                    curr = self.get_node_by_ref(r#ref)?;
                }
                Some(curr)
            }
        }
    }

    pub fn insert_input(&mut self, name: &str, input: InputNode) -> InputNodeRef {
        let r#ref = self.insert_node(name, input);
        self.input_refs_mut()
            .insert(name.to_owned(), r#ref.clone())
            .unwrap();
        r#ref
    }

    pub fn insert_node(&mut self, name: &str, input: InputNode) -> InputNodeRef {
        let name = {
            let mut i = 1;
            let mut new_name = name.to_owned();
            while self.nodes.contains_key(&new_name) {
                i += 1;
                new_name = format!("{name}_{i}");
            }
            new_name
        };
        self.nodes.insert(name.clone(), input);
        InputNodeRef::Name(name)
    }

    pub fn copy_node_from(
        &mut self,
        other: &Self,
        r#ref: &InputNodeRef,
        name_base: impl AsRef<str>,
    ) -> Result<InputNodeRef, ()> {
        let mut node = other.get_node_by_ref(r#ref).ok_or(())?.clone();
        if let Some(inputs) = &mut node.inputs {
            for (input_name, input_ref) in inputs {
                *input_ref = self.copy_node_from(other, input_ref, input_name)?;
            }
        }
        Ok(self.insert_node(name_base.as_ref(), node))
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
