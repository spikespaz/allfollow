mod flake_lock;

use std::collections::HashMap;
use std::ops::Deref;

use flake_lock::{LockFile, LockedNode, NodeEdge, MAX_SUPPORTED_LOCK_VERSION};

fn main() {
    let input_lock = std::fs::read_to_string("./samples/hyprnix/before/flake.lock")
        .expect("samples/a/flake.lock does not exist");

    let old_lock: LockFile = {
        let deser = &mut serde_json::Deserializer::from_str(&input_lock);
        match serde_path_to_error::deserialize(deser) {
            Ok(lock) => lock,
            Err(e) => panic!("{}", e),
        }
    };

    // if old_lock.version != MAX_SUPPORTED_LOCK_VERSION {
    //     panic!("This program supports flake lock files of schema version {} while the flake you have asked to modify is of version {}", MAX_SUPPORTED_LOCK_VERSION, old_lock.version)
    // }

    dbg!(&old_lock);

    // let mut flake_inputs = old_lock.input_refs().clone();
    // let mut new_lock = FlakeLock::new();

    // for (name, r#ref) in &mut flake_inputs {
    //     *r#ref = new_lock.copy_node_from(&old_lock, r#ref, name).unwrap();
    // }

    // println!("{}", serde_json::to_string_pretty(&new_lock).unwrap());
}
