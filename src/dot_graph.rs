use dot_generator::{attr, edge, graph, id, node, node_id};
use dot_structures::{Attribute, Edge, EdgeTy, Graph, Id, Node, NodeId, Stmt, Vertex};

use crate::flake_lock::LockFile;

impl TryFrom<LockFile> for Graph {
    type Error = ();

    fn try_from(lock: LockFile) -> Result<Self, Self::Error> {
        let mut graph = graph!(strict di id!("flakelock"));

        let nodes = lock
            .node_indices()
            .map(|index| (index, lock.get_node(index).expect("a node to exist")));

        for (index, node) in nodes {
            graph.add_stmt(Stmt::Node(node!(index; attr!("shape", "circle"))));
            for (_name, edge) in node.iter_edges() {
                let res = lock.resolve_edge(&edge).expect("resolution");
                graph.add_stmt(Stmt::Edge(edge!(node_id!(index) => node_id!(&res))));
            }
        }

        Ok(graph)
    }
}

// fn recurse_inputs(lock: &LockFile, index: String, op: &mut impl FnMut(String)) {
//     let node = lock.get_node(&index).unwrap();
//     op(index);
//     for (name, edge) in node.iter_edges() {
//         let index = lock.resolve_edge(&edge).unwrap();
//         recurse_inputs(lock, index, op);
//     }
// }
