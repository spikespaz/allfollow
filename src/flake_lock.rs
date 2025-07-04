use std::cell::{Ref, RefCell, RefMut};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub const MAX_SUPPORTED_LOCK_VERSION: u32 = 7;
pub const MIN_SUPPORTED_LOCK_VERSION: u32 = 5;

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LockFile {
    nodes: IndexMap<String, RefCell<Node>>,
    root: String,
    version: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, untagged)]
pub enum NodeEdge {
    Indexed(String),
    Follows(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, untagged)]
pub enum Node {
    Locked(LockedNode),
    Unlocked(UnlockedNode),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LockedNode {
    #[serde(skip_serializing_if = "Clone::clone", default = "default_true")]
    flake: bool,
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    inputs: IndexMap<String, RefCell<NodeEdge>>,
    locked: serde_json::Value,
    original: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UnlockedNode {
    inputs: IndexMap<String, RefCell<NodeEdge>>,
}

impl NodeEdge {
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

impl std::fmt::Display for NodeEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Indexed(index) => write!(f, "{index}"),
            Self::Follows(path) => {
                let mut path = path.iter().peekable();
                while let Some(name) = path.next() {
                    write!(f, "{name}")?;
                    if path.peek().is_some() {
                        write!(f, "/")?;
                    }
                }
                Ok(())
            }
        }
    }
}

pub trait NodeEdgeRef<'a> {
    fn index(self) -> Option<Ref<'a, str>>;

    #[expect(unused)]
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

impl<A> FromIterator<A> for NodeEdge
where
    A: AsRef<str>,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self::Follows(iter.into_iter().map(|s| s.as_ref().to_string()).collect())
    }
}

impl Node {
    fn edges(&self) -> &IndexMap<String, RefCell<NodeEdge>> {
        match self {
            Self::Locked(LockedNode { inputs, .. }) => inputs,
            Self::Unlocked(UnlockedNode { inputs }) => inputs,
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

    #[expect(unused)]
    pub fn get_edge_mut(&self, name: impl AsRef<str>) -> Option<RefMut<NodeEdge>> {
        self.edges()
            .get(name.as_ref())
            .map(|cell| cell.borrow_mut())
    }
}

impl LockFile {
    #[expect(unused)]
    pub fn new() -> Self {
        static ROOT: &str = "root";
        Self {
            nodes: IndexMap::from_iter([(
                ROOT.into(),
                RefCell::new(Node::Unlocked(UnlockedNode {
                    inputs: IndexMap::new(),
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

    #[expect(unused)]
    pub fn get_node_mut(&self, index: impl AsRef<str>) -> Option<RefMut<Node>> {
        self.nodes.get(index.as_ref()).map(RefCell::borrow_mut)
    }

    pub fn remove_node(&mut self, index: impl AsRef<str>) -> Option<Node> {
        self.nodes
            .swap_remove(index.as_ref())
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
