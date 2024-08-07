mod flake_lock;

use flake_lock::FlakeLock;

const SUPPORTED_LOCK_VERSION: u32 = 7;

fn main() {
    let input_lock = std::fs::read_to_string("./samples/hyprnix/before/flake.lock")
        .expect("samples/a/flake.lock does not exist");

    let lock: FlakeLock = {
        let deser = &mut serde_json::Deserializer::from_str(&input_lock);
        match serde_path_to_error::deserialize(deser) {
            Ok(lock) => lock,
            Err(e) => panic!("{}", e),
        }
    };

    if lock.version != SUPPORTED_LOCK_VERSION {
        panic!("This program supports flake lock files of schema version {} while the flake you have asked to modify is of version {}", SUPPORTED_LOCK_VERSION, lock.version)
    }

    dbg!(lock);
}
