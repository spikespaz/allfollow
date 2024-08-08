use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const MAX_SUPPORTED_LOCK_VERSION: u32 = 7;
pub const MIN_SUPPORTED_LOCK_VERSION: u32 = 5;

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockFile {
    nodes: HashMap<String, Node>,
    root: String,
    version: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeEdge {
    Indexed(String),
    Follows(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Node {
    Unlocked(UnlockedNode),
    Locked(LockedNode),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnlockedNode {
    inputs: HashMap<String, NodeEdge>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedNode {
    #[serde(skip_serializing_if = "Clone::clone", default = "default_true")]
    flake: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    inputs: Option<HashMap<String, NodeEdge>>,
    locked: Box<LockedReference>,
    original: Box<FlakeReference>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedReference {
    last_modified: usize,
    nar_hash: String,
    #[serde(flatten)]
    flake_ref: FlakeReference,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum FlakeReference {
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

impl NodeEdge {
    pub fn from_iter(iter: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self::Follows(iter.into_iter().map(|s| s.as_ref().to_string()).collect())
    }
}

impl From<&str> for NodeEdge {
    fn from(value: &str) -> Self {
        Self::Indexed(value.to_string())
    }
}

impl From<String> for NodeEdge {
    fn from(value: String) -> Self {
        Self::Indexed(value)
    }
}

impl From<Vec<String>> for NodeEdge {
    fn from(value: Vec<String>) -> Self {
        Self::Follows(value)
    }
}

impl LockFile {
    pub fn new() -> Self {
        static ROOT: &str = "root";
        Self {
            nodes: HashMap::from_iter([(
                ROOT.into(),
                Node::Unlocked(UnlockedNode {
                    inputs: HashMap::new(),
                }),
            )]),
            root: ROOT.into(),
            version: MAX_SUPPORTED_LOCK_VERSION,
        }
    }

    // pub fn root(&self) -> &InputNode {
    //     self.nodes
    //         .get(&self.root)
    //         .expect("the root node to already exist")
    // }

    // fn root_mut(&mut self) -> &mut InputNode {
    //     self.nodes
    //         .get_mut(&self.root)
    //         .expect("the root node to already exist")
    // }

    // pub fn input_refs(&self) -> &HashMap<String, InputNodeRef> {
    //     self.root()
    //         .inputs
    //         .as_ref()
    //         .expect("the root node to have been initialized")
    // }

    // fn input_refs_mut(&mut self) -> &mut HashMap<String, InputNodeRef> {
    //     self.root_mut()
    //         .inputs
    //         .as_mut()
    //         .expect("the root node to have been initialized")
    // }

    // pub fn get_input_by_name(&self, name: impl AsRef<str>) -> Option<&InputNode> {
    //     self.input_refs()
    //         .get(name.as_ref())
    //         .and_then(|r#ref| self.get_node_by_ref(r#ref))
    // }

    // pub fn get_node_by_ref(&self, r#ref: &InputNodeRef) -> Option<&InputNode> {
    //     match r#ref {
    //         InputNodeRef::Name(name) => self.nodes.get(name),
    //         InputNodeRef::Follows(path) => {
    //             let mut curr = self.get_input_by_name(path.first()?)?;
    //             for name in path.iter().skip(1) {
    //                 let r#ref = curr.inputs.as_ref()?.get(name)?;
    //                 curr = self.get_node_by_ref(r#ref)?;
    //             }
    //             Some(curr)
    //         }
    //     }
    // }

    // pub fn insert_input(&mut self, name: &str, input: InputNode) -> InputNodeRef {
    //     let r#ref = self.insert_node(name, input);
    //     self.input_refs_mut()
    //         .insert(name.to_owned(), r#ref.clone())
    //         .unwrap();
    //     r#ref
    // }

    // pub fn insert_node(&mut self, name: &str, input: InputNode) -> InputNodeRef {
    //     let name = {
    //         let mut i = 1;
    //         let mut new_name = name.to_owned();
    //         while self.nodes.contains_key(&new_name) {
    //             i += 1;
    //             new_name = format!("{name}_{i}");
    //         }
    //         new_name
    //     };
    //     self.nodes.insert(name.clone(), input);
    //     InputNodeRef::Name(name)
    // }

    // pub fn copy_node_from(
    //     &mut self,
    //     other: &Self,
    //     r#ref: &InputNodeRef,
    //     name_base: impl AsRef<str>,
    // ) -> Result<InputNodeRef, ()> {
    //     let mut node = other.get_node_by_ref(r#ref).ok_or(())?.clone();
    //     if let Some(inputs) = &mut node.inputs {
    //         for (input_name, input_ref) in inputs {
    //             *input_ref = self.copy_node_from(other, input_ref, input_name)?;
    //         }
    //     }
    //     Ok(self.insert_node(name_base.as_ref(), node))
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::FlakeLock;

//     macro_rules! parse_lock_file {
//         ($test_ident:ident, $lock_file_path:literal) => {
//             #[test]
//             fn $test_ident() {
//                 static SOURCE: &str = include_str!($lock_file_path);
//                 let deser = &mut serde_json::Deserializer::from_str(SOURCE);
//                 let res: Result<FlakeLock, _> = serde_path_to_error::deserialize(deser);
//                 match res {
//                     Ok(lock) => {
//                         dbg!(lock);
//                     }
//                     Err(e) => panic!("{}", e),
//                 }
//             }
//         };
//     }

//     parse_lock_file!(parse_hyprnix_before, "../samples/hyprnix/before/flake.lock");
//     parse_lock_file!(parse_hyprnix_after, "../samples/hyprnix/after/flake.lock");
// }
