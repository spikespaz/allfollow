use dot_generator::{attr, edge, graph, id, node, node_id, stmt};
use dot_structures::{Attribute, Edge, EdgeTy, Graph, Id, Node, NodeId, Stmt, Vertex};
use heck::ToSnakeCase;

use crate::flake_lock::LockFile;

impl TryFrom<LockFile> for Graph {
    type Error = ();

    fn try_from(lock: LockFile) -> Result<Self, Self::Error> {
        let mut graph = graph!(strict di id!("flakelock"));

        let nodes = lock
            .node_indices()
            .map(|index| (index, lock.get_node(index).expect("a node to exist")));

        for (index, node) in nodes {
            let node_id = if index == "root" {
                "flake_root".into()
            } else {
                index.to_snake_case()
            };
            graph.add_stmt(stmt!(node!(node_id; attr!("shape", "circle"))));
            for (input_name, edge) in node.iter_edges() {
                let input_node_index = lock.resolve_edge(&edge).expect("resolution");
                let input_node_id = input_node_index.to_snake_case();
                graph.add_stmt(stmt!(edge!(node_id!(node_id) => node_id!(input_node_id))));
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
