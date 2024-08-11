mod flake_lock;

use std::collections::HashMap;
use std::iter::repeat;

use flake_lock::{
    LockFile, Node, NodeEdgeRef as _, MAX_SUPPORTED_LOCK_VERSION, MIN_SUPPORTED_LOCK_VERSION,
};

fn main() {
    let file_content = std::fs::read_to_string("./samples/hyprnix/before/flake.lock")
        .expect("samples/a/flake.lock does not exist");

    let lock: LockFile = {
        let deser = &mut serde_json::Deserializer::from_str(&file_content);
        match serde_path_to_error::deserialize(deser) {
            Ok(lock) => lock,
            Err(e) => panic!("{}", e),
        }
    };

    if lock.version() < MIN_SUPPORTED_LOCK_VERSION && lock.version() > MAX_SUPPORTED_LOCK_VERSION {
        panic!(
            "This program supports lock files between schema versions {} and {} while the flake you have asked to modify is of version {}",
            MIN_SUPPORTED_LOCK_VERSION,
            MAX_SUPPORTED_LOCK_VERSION,
            lock.version()
        );
    }

    let root = &*lock.root();

    for index in root.iter_edges().filter_map(|(_, edge)| edge.index()) {
        let input = &*lock.get_node(&*index).unwrap();
        for (name, mut edge) in input.iter_edges_mut() {
            if let Some(root_edge) = root.get_edge(name) {
                *edge = (*root_edge).clone();
            }
        }
    }

    fn recurse_inputs(lock: &LockFile, node: &Node, op: &mut impl FnMut(String)) {
        for (_, edge) in node.iter_edges() {
            let index = lock.resolve_edge(&edge).unwrap();
            let next_node = &*lock.get_node(&index).unwrap();
            op(index);
            recurse_inputs(lock, next_node, op);
        }
    }

    let mut node_hits = HashMap::<_, _>::from_iter(lock.node_indices().zip(repeat(0_u32)));
    recurse_inputs(&lock, root, &mut |index| {
        *node_hits.get_mut(index.as_str()).unwrap() += 1;
    });

    println!("{}", serde_json::to_string(&lock).unwrap());
    println!("{:#?}", node_hits);
}
