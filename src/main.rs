mod flake_lock;

use std::collections::HashMap;
use std::iter::repeat;

use flake_lock::{
    LockFile, NodeEdgeRef as _, MAX_SUPPORTED_LOCK_VERSION, MIN_SUPPORTED_LOCK_VERSION,
};

fn recurse_inputs(lock: &LockFile, index: impl AsRef<str>, op: &mut impl FnMut(&str)) {
    op(index.as_ref());
    for (_, edge) in lock.get_node(index).unwrap().iter_edges() {
        let index = lock.resolve_edge(&edge).unwrap();
        recurse_inputs(lock, index, op);
    }
}

fn main() {
    let file_content = std::fs::read_to_string("./samples/hyprnix/before/flake.lock")
        .expect("samples/a/flake.lock does not exist");

    let lock: LockFile = {
        let deserializer = &mut serde_json::Deserializer::from_str(&file_content);
        serde_path_to_error::deserialize(deserializer).unwrap_or_else(|e| panic!("{}", e))
    };

    if lock.version() < MIN_SUPPORTED_LOCK_VERSION && lock.version() > MAX_SUPPORTED_LOCK_VERSION {
        panic!(
            "This program supports lock files between schema versions {} and {} while the flake you have asked to modify is of version {}",
            MIN_SUPPORTED_LOCK_VERSION,
            MAX_SUPPORTED_LOCK_VERSION,
            lock.version()
        );
    }

    let root = lock.root().unwrap();

    for index in root.iter_edges().filter_map(|(_, edge)| edge.index()) {
        let input = &*lock.get_node(&*index).unwrap();
        for (name, mut edge) in input.iter_edges_mut() {
            if let Some(root_edge) = root.get_edge(name) {
                *edge = (*root_edge).clone();
            }
        }
    }

    let mut node_hits = HashMap::<_, _>::from_iter(lock.node_indices().zip(repeat(0_u32)));
    recurse_inputs(&lock, lock.root_index(), &mut |index| {
        *node_hits.get_mut(index).unwrap() += 1;
    });

    let dead_nodes = node_hits
        .into_iter()
        .filter(|(_, hits)| *hits == 0)
        .map(|(index, _)| index.to_string())
        .collect::<Vec<_>>();

    drop(root);

    let mut lock = lock;
    for index in dead_nodes {
        lock.remove_node(index);
    }

    println!("{}", serde_json::to_string(&lock).unwrap());
}
