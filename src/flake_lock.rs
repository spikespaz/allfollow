use std::cell::{Ref, RefCell, RefMut};
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
    nodes: HashMap<String, RefCell<Node>>,
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
    inputs: HashMap<String, RefCell<NodeEdge>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedNode {
    #[serde(skip_serializing_if = "Clone::clone", default = "default_true")]
    flake: bool,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    inputs: HashMap<String, RefCell<NodeEdge>>,
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

    pub fn index(&self) -> Option<&str> {
        match self {
            Self::Indexed(index) => Some(index.as_str()),
            _ => None,
        }
    }

    pub fn path(&self) -> Option<&Vec<String>> {
        match self {
            Self::Follows(path) => Some(path),
            _ => None,
        }
    }
}

pub trait NodeEdgeRef<'a> {
    fn index(self) -> Option<Ref<'a, str>>;

    fn path(self) -> Option<Ref<'a, Vec<String>>>;
}

impl<'a> NodeEdgeRef<'a> for Ref<'a, NodeEdge> {
    fn index(self) -> Option<Ref<'a, str>> {
        Ref::filter_map(self, NodeEdge::index).ok()
    }

    fn path(self) -> Option<Ref<'a, Vec<String>>> {
        Ref::filter_map(self, NodeEdge::path).ok()
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

impl Node {
    fn edges(&self) -> &HashMap<String, RefCell<NodeEdge>> {
        match self {
            Self::Unlocked(UnlockedNode { inputs }) => inputs,
            Self::Locked(LockedNode { inputs, .. }) => inputs,
        }
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = (&str, Ref<NodeEdge>)> {
        self.edges()
            .iter()
            .map(|(name, edge)| (name.as_str(), edge.borrow()))
    }

    pub fn iter_edges_mut(&self) -> impl Iterator<Item = (&str, RefMut<NodeEdge>)> {
        self.edges()
            .iter()
            .map(|(name, edge)| (name.as_str(), edge.borrow_mut()))
    }

    pub fn get_edge(&self, name: impl AsRef<str>) -> Option<Ref<NodeEdge>> {
        self.edges().get(name.as_ref()).map(|cell| cell.borrow())
    }

    pub fn get_edge_mut(&self, name: impl AsRef<str>) -> Option<RefMut<NodeEdge>> {
        self.edges()
            .get(name.as_ref())
            .map(|cell| cell.borrow_mut())
    }

    // pub fn iter_inputs<'lock>(
    //     &'lock self,
    //     lock: &'lock LockFile,
    // ) -> impl Iterator<Item = (&str, Option<(&str, Ref<Node>)>)> {
    //     self.edges()
    //         .iter()
    //         .map(|(name, edge)| (name.as_str(), lock.resolve_edge(edge)))
    // }
}

impl LockFile {
    pub fn new() -> Self {
        static ROOT: &str = "root";
        Self {
            nodes: HashMap::from_iter([(
                ROOT.into(),
                RefCell::new(Node::Unlocked(UnlockedNode {
                    inputs: HashMap::new(),
                })),
            )]),
            root: ROOT.into(),
            version: MAX_SUPPORTED_LOCK_VERSION,
        }
    }

    pub fn root(&self) -> Option<Ref<Node>> {
        self.nodes.get(&self.root).map(RefCell::borrow)
    }

    pub fn root_index(&self) -> &str {
        &self.root
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn node_indices(&self) -> impl Iterator<Item = &str> {
        self.nodes.keys().map(String::as_str)
    }

    pub fn get_node(&self, index: impl AsRef<str>) -> Option<Ref<Node>> {
        self.nodes.get(index.as_ref()).map(RefCell::borrow)
    }

    pub fn get_node_mut(&self, index: impl AsRef<str>) -> Option<RefMut<Node>> {
        self.nodes.get(index.as_ref()).map(RefCell::borrow_mut)
    }

    pub fn remove_node(&mut self, index: impl AsRef<str>) -> Option<Node> {
        self.nodes
            .remove(index.as_ref())
            .map(|cell| cell.into_inner())
    }

    pub fn resolve_edge(&self, edge: &NodeEdge) -> Option<String> {
        match edge {
            NodeEdge::Indexed(index) => Some(index.to_owned()),
            NodeEdge::Follows(path) => self.follow_path(path),
        }
    }

    pub fn follow_path(&self, path: impl IntoIterator<Item = impl AsRef<str>>) -> Option<String> {
        path.into_iter().try_fold(self.root.clone(), |index, name| {
            self.resolve_edge(&*self.get_node(index)?.get_edge(name)?)
        })
    }
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
