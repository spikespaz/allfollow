mod cli_args;
mod flake_lock;
mod fmt_colors;

use std::collections::HashMap;
use std::iter::repeat;

use bpaf::Bpaf;
use cli_args::{Input, Output};
use flake_lock::{
    LockFile, NodeEdge, NodeEdgeRef as _, MAX_SUPPORTED_LOCK_VERSION, MIN_SUPPORTED_LOCK_VERSION,
};
use owo_colors::OwoColorize;
use serde::Serialize;
use serde_json::Serializer;

/// Imitate Nix flake input following behavior as a post-process,
/// so that you can stop manually maintaining tedious connections
/// between many flake inputs.
/// This small tool aims to replace every instance of
/// `inputs.*.inputs.*.follows = "*";` in your `flake.nix` with automation.
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, generate(parse_env_args))]
struct EnvArgs {
    /// Write new lock file back to the source
    #[bpaf(short('I'), long)]
    pub in_place: bool,
    /// Overwrite the output file if it exists
    #[bpaf(short('f'), long, long("force"))]
    pub overwrite: bool,
    /// Do not minify the output JSON
    #[bpaf(short('p'), long)]
    pub pretty: bool,
    /// Do not imitate `inputs.*.follows`, reference node indices instead
    #[bpaf(long, long("indexed"))]
    pub no_follows: bool,
    /// Path of the new lock file to write, set to `-` for stdout (default)
    #[bpaf(short('o'), long, argument("OUTPUT"), fallback(Output::Stdout))]
    pub output: Output,
    /// The path of `flake.lock` to read, or `-` to read from standard input.
    /// If unspecified, defaults to the current directory.
    #[bpaf(positional("INPUT"), fallback(Input::from("./flake.lock")))]
    pub lock_file: Input,
}

impl EnvArgs {
    fn from_env() -> Self {
        let mut args = parse_env_args().run();
        if args.in_place {
            args.output = Output::from(args.lock_file.clone());
            args.overwrite = true;
        }
        args
    }
}

fn main() {
    let args = EnvArgs::from_env();

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

    let node_hits = FlakeNodeVisits::count_from_index(&lock, lock.root_index());
    eprintln!();
    elogln!(:bold :bright_magenta "Flake input nodes' reference counts:"; &node_hits);

    let root = lock.root().unwrap();

    elogln!(:bold :bright_magenta "Redirecting inputs to imitate follows behavior.");
    for (input_name, index) in root
        .iter_edges()
        .filter_map(|(name, edge)| edge.index().map(|index| (name, index)))
    {
        let input = &*lock.get_node(&*index).unwrap();
        for (edge_name, mut edge) in input.iter_edges_mut() {
            if let Some(root_edge) = root.get_edge(edge_name) {
                if args.no_follows {
                    *edge = (*root_edge).clone()
                } else {
                    *edge = NodeEdge::from_iter([edge_name])
                }
                // TODO differentiate between indices (green) and follows (yellow)
                elogln!("Redirected input", :bold (:yellow "'{input_name}/{edge_name}'", :bright_white "->", :green "'{edge}'"));
            } else {
                elogln!(
                    :bold (:cyan "No suitable replacement for", :yellow "'{input_name}/{edge_name}'"),
                    :dimmed "(" :italic ("'" (lock.resolve_edge(&edge).unwrap()) "'") :dimmed ")"
                    // bug in owo-color
                    // :dimmed ("(" :italic ("'" (lock.resolve_edge(&edge).unwrap()) "'") ")")
                );
            }
        }
    }

    drop(root);

    let node_hits = FlakeNodeVisits::count_from_index(&lock, lock.root_index());
    let dead_nodes = node_hits
        .into_inner()
        .into_iter()
        .filter(|&(_, count)| count == 0)
        .map(|(index, _)| index.to_owned())
        .collect::<Vec<_>>();

    let mut lock = lock;
    for index in dead_nodes {
        lock.remove_node(&index);
        elogln!("Pruned orphan", :bold :red "'{index}'");
    }

    eprintln!();
    elog!(
        :bold (:bright_magenta "Flake input nodes' reference counts", :bright_green "after successful pruning" :bright_magenta ":");
        (FlakeNodeVisits::count_from_index(&lock, lock.root_index()))
    );
    eprintln!();

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

fn recurse_inputs(lock: &LockFile, index: String, op: &mut impl FnMut(String)) {
    let node = lock.get_node(&index).unwrap();
    op(index);
    for (_, edge) in node.iter_edges() {
        let index = lock.resolve_edge(&edge).unwrap();
        recurse_inputs(lock, index, op);
    }
}

struct FlakeNodeVisits<'a> {
    inner: HashMap<&'a str, u32>,
    // Index of the node which this count is relative to.
    root_index: &'a str,
}

impl<'a> FlakeNodeVisits<'a> {
    fn count_from_index<'new>(lock: &'new LockFile, index: &'new str) -> FlakeNodeVisits<'new> {
        let mut node_hits = HashMap::from_iter(lock.node_indices().zip(repeat(0_u32)));
        recurse_inputs(lock, index.to_owned(), &mut |index| {
            *node_hits.get_mut(index.as_str()).unwrap() += 1;
        });
        FlakeNodeVisits {
            inner: node_hits,
            root_index: index,
        }
    }

    fn into_inner(self) -> HashMap<&'a str, u32> {
        self.inner
    }
}

impl<'a> From<FlakeNodeVisits<'a>> for HashMap<&'a str, u32> {
    fn from(value: FlakeNodeVisits<'a>) -> Self {
        value.into_inner()
    }
}

impl<'a> std::ops::Deref for FlakeNodeVisits<'a> {
    type Target = HashMap<&'a str, u32>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> std::ops::DerefMut for FlakeNodeVisits<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a> std::fmt::Display for FlakeNodeVisits<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let max_pad = {
            let (mut min_len, mut max_len) = (0, 0);
            for key in self.inner.keys() {
                min_len = std::cmp::min(min_len, key.len());
                max_len = std::cmp::max(max_len, key.len());
            }
            max_len - min_len
        };
        for (index, count) in self.inner.iter() {
            if index == &self.root_index {
                f.write_fmt(format_args_colored!(
                    :dimmed .("{:1$}", index, max_pad), :red "=", :dimmed &count;
                ))?
            } else if *count <= 1 {
                f.write_fmt(format_args_colored!(
                    :bold :bright_yellow .("{:1$}", index, max_pad), :red "=", :dimmed &count;
                ))?
            } else {
                f.write_fmt(format_args_colored!(
                    .("{:1$}", index, max_pad), :red "=", :bold :bright_green &count;
                ))?
            }
        }
        Ok(())
    }
}
