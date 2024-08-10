mod flake_lock;

use flake_lock::{LockFile, MAX_SUPPORTED_LOCK_VERSION, MIN_SUPPORTED_LOCK_VERSION};

use self::flake_lock::NodeEdge;

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

    let root = lock.root();

    for (_, edge) in root.iter_edges() {
        let NodeEdge::Indexed(index) = &*edge else {
            continue;
        };
        let input = &*lock.get_node(index).unwrap();
        for (name, mut edge) in input.iter_edges_mut() {
            let Some(root_edge) = root.get_edge(name) else {
                continue;
            };
            *edge = (*root_edge).clone();
        }
    }
}
