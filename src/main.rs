mod flake_lock;

use std::collections::HashMap;
use std::ops::Deref;

use flake_lock::{
    LockFile, LockedNode, NodeEdge, MAX_SUPPORTED_LOCK_VERSION, MIN_SUPPORTED_LOCK_VERSION,
};

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

    if old_lock.version() < MIN_SUPPORTED_LOCK_VERSION
        && old_lock.version() > MAX_SUPPORTED_LOCK_VERSION
    {
        panic!(
            "This program supports lock files between schema versions {} and {} while the flake you have asked to modify is of version {}",
            MIN_SUPPORTED_LOCK_VERSION,
            MAX_SUPPORTED_LOCK_VERSION,
            old_lock.version()
        );
    }

    dbg!(old_lock
        .root()
        .follow_path(&old_lock, ["hyprland", "aquamarine", "systems"]));

    for (name, pair) in old_lock.root().iter_inputs(&old_lock) {
        let Some((index, node)) = pair else {
            panic!("root node has dangling input `{name}`");
        };
        println!(
            "root node has input named `{name}`, which is found by the index `{index}`\n{node:?}"
        );
    }

    // let mut flake_inputs = old_lock.input_refs().clone();
    // let mut new_lock = FlakeLock::new();

    // for (name, r#ref) in &mut flake_inputs {
    //     *r#ref = new_lock.copy_node_from(&old_lock, r#ref, name).unwrap();
    // }

    // println!("{}", serde_json::to_string_pretty(&new_lock).unwrap());
}
