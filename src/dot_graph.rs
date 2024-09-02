use std::borrow::Cow;

use dotwalk as dot;

use crate::flake_lock::{LockFile, NodeEdge};

type Node<'a> = Cow<'a, str>;

#[derive(Clone, Debug, PartialEq)]
pub struct Edge<'a> {
    pub source: Node<'a>,
    pub name: Cow<'a, str>,
    pub inner: Cow<'a, NodeEdge>,
}

impl<'a> dot::GraphWalk<'a> for LockFile {
    type Node = Node<'a>;
    type Edge = Edge<'a>;
    type Subgraph = ();

    fn nodes(&'a self) -> dot::Nodes<'a, Self::Node> {
        self.node_indices()
            .map(Into::into)
            .collect::<Vec<_>>()
            .into()
    }

    fn edges(&'a self) -> dotwalk::Edges<'a, Self::Edge> {
        let mut edges = Vec::new();
        for (index, input) in self.iter_nodes() {
            for (name, edge) in input.iter_edges() {
                edges.push(Edge {
                    source: index.into(),
                    name: name.to_owned().into(),
                    inner: Cow::Owned(edge.clone()),
                })
            }
        }
        edges.into()
    }

    fn source(&'a self, edge: &Self::Edge) -> Self::Node {
        edge.source.clone()
    }

    fn target(&'a self, edge: &Self::Edge) -> Self::Node {
        self.resolve_edge(edge.inner.as_ref())
            .expect("a node to exist")
            .into()
    }
}

impl<'a> dot::Labeller<'a> for LockFile {
    type Node = Node<'a>;
    type Edge = Edge<'a>;
    type Subgraph = ();

    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("flakelock").unwrap()
    }

    fn node_id(&'a self, n: &Self::Node) -> dot::Id<'a> {
        dot::Id::new(n.replace('-', "_")).expect("node index is not valid graphviz Id")
    }

    fn node_label(&'a self, n: &Self::Node) -> dot::Text<'a> {
        dot::Text::label(n.clone())
    }
}
