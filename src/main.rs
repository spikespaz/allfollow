mod cli_args;
mod flake_lock;

use std::collections::HashMap;
use std::iter::repeat;

use argh::FromArgs;
use cli_args::{Input, Output};
use flake_lock::{
    LockFile, NodeEdgeRef as _, MAX_SUPPORTED_LOCK_VERSION, MIN_SUPPORTED_LOCK_VERSION,
};
use serde::Serialize;
use serde_json::Serializer;

#[derive(FromArgs)]
#[argh(
    description = "Automatically redirect top-level flake inputs' edges to follow all other identically named top-level inputs."
)]
struct Args {
    #[argh(
        positional,
        default = r#"Input::from("./flake.lock")"#,
        description = "path of the flake lock to read, set to `-` to read from stdin"
    )]
    pub lock_file: Input,
    #[argh(
        switch,
        short = 'i',
        description = "set the output path to the same as the original lock and overwrite with no confirmation"
    )]
    pub in_place: bool,
    #[argh(
        option,
        short = 'o',
        default = r#"Output::Stdout"#,
        description = "path of the output file, set to `-` to print to stdout (default)"
    )]
    pub output: Output,
    #[argh(
        switch,
        short = 'f',
        description = "overwrite the output file if it already exists"
    )]
    pub overwrite: bool,
    #[argh(switch, short = 'p', description = "serialize pretty output")]
    pub pretty: bool,
}

impl Args {
    fn from_env() -> Self {
        let mut args = argh::from_env::<Args>();
        if args.in_place {
            args.output = Output::from(args.lock_file.clone());
            args.overwrite = true;
        }
        args
    }
}

fn main() {
    let args = Args::from_env();

    let reader = args
        .lock_file
        .open()
        .unwrap_or_else(|e| panic!("Failed to read the input file: {e}"));
    let deserializer = &mut serde_json::Deserializer::from_reader(reader);
    let lock: LockFile = {
        serde_path_to_error::deserialize(deserializer)
            .unwrap_or_else(|e| panic!("Failed to deserialize the provided flake lock: {e}"))
    };

    if lock.version() < MIN_SUPPORTED_LOCK_VERSION && lock.version() > MAX_SUPPORTED_LOCK_VERSION {
        panic!(
            "This program supports lock files between schema versions {} and {} while the flake you have asked to modify is of version {}.",
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

    let writer = args
        .output
        .create(!args.overwrite)
        .unwrap_or_else(|e| panic!("Could not write to output: {e}"));

    let res = if args.pretty {
        lock.serialize(&mut Serializer::pretty(writer))
    } else {
        lock.serialize(&mut Serializer::new(writer))
    };

    if let Err(e) = res {
        panic!("Failed while serializing to output, file is probably corrupt: {e}")
    }
}

fn recurse_inputs(lock: &LockFile, index: impl AsRef<str>, op: &mut impl FnMut(&str)) {
    op(index.as_ref());
    for (_, edge) in lock.get_node(index).unwrap().iter_edges() {
        let index = lock.resolve_edge(&edge).unwrap();
        recurse_inputs(lock, index, op);
    }
}
