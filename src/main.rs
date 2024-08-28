mod cli_args;
mod flake_lock;
mod fmt_colors;

use std::collections::HashMap;
use std::iter::repeat;

use bpaf::Bpaf;
use cli_args::{Input, Output};
use flake_lock::{
    LockFile, Node, NodeEdge, NodeEdgeRef as _, MAX_SUPPORTED_LOCK_VERSION,
    MIN_SUPPORTED_LOCK_VERSION,
};
use owo_colors::OwoColorize;
use serde::Serialize;
use serde_json::Serializer;

static EXPECT_ROOT_EXIST: &str = "the root node to exist";

/// Imitate Nix flake input following behavior as a post-process,
/// so that you can stop manually maintaining tedious connections
/// between many flake inputs.
/// This small tool aims to replace every instance of
/// `inputs.*.inputs.*.follows = "*";` in your `flake.nix` with automation.
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, generate(parse_command_env_args))]
enum Command {
    #[bpaf(command("prune"))]
    Prune {
        /// Do not imitate `inputs.*.follows`, reference node indices instead
        #[bpaf(long, long("indexed"))]
        no_follows: bool,
        //
        #[bpaf(external(json_output_options))]
        output_opts: JsonOutputOptions,
        /// The path of `flake.lock` to read, or `-` to read from standard input.
        /// If unspecified, defaults to the current directory.
        #[bpaf(positional("INPUT"), fallback(Input::from("./flake.lock")))]
        lock_file: Input,
    },
    #[bpaf(command("count"))]
    Count {
        /// Show the data as JSON.
        #[bpaf(short('j'), long)]
        json: bool,
        //
        #[bpaf(external(json_output_options))]
        output_opts: JsonOutputOptions,
        /// The path of `flake.lock` to read, or `-` to read from standard input.
        /// If unspecified, defaults to the current directory.
        #[bpaf(positional("INPUT"), fallback(Input::from("./flake.lock")))]
        lock_file: Input,
    },
}

/// Options for output handling:
#[derive(Debug, Clone, Bpaf)]
struct JsonOutputOptions {
    /// Write new lock file back to the source
    #[bpaf(short('I'), long)]
    in_place: bool,
    /// Overwrite the output file if it exists
    #[bpaf(short('f'), long, long("force"))]
    overwrite: bool,
    /// Do not minify the output JSON
    #[bpaf(short('p'), long)]
    pretty: bool,
    /// Path of the new lock file to write, set to `-` for stdout (default)
    #[bpaf(short('o'), long, argument("OUTPUT"), fallback(Output::Stdout))]
    output: Output,
}

impl Command {
    fn from_env() -> Self {
        let mut args = parse_command_env_args().run();
        #[allow(clippy::single_match)]
        match &mut args {
            Command::Prune {
                lock_file,
                output_opts:
                    JsonOutputOptions {
                        in_place,
                        overwrite,
                        pretty: _,
                        output,
                    },
                ..
            }
            | Command::Count {
                lock_file,
                output_opts:
                    JsonOutputOptions {
                        in_place,
                        overwrite,
                        pretty: _,
                        output,
                    },
                ..
            } => {
                if *in_place {
                    *output = Output::from(lock_file.clone());
                    *overwrite = true;
                }
            }
        };
        args
    }
}

fn main() {
    match Command::from_env() {
        Command::Prune {
            no_follows,
            lock_file,
            output_opts:
                JsonOutputOptions {
                    in_place: _,
                    overwrite,
                    pretty,
                    output,
                },
        } => {
            let mut lock = read_flake_lock(lock_file);

            let node_hits = FlakeNodeVisits::count_from_index(&lock, lock.root_index());
            eprintln!();
            elogln!(:bold :bright_magenta "Flake input nodes' reference counts:"; &node_hits);

            substitute_flake_inputs_with_follows(&lock, no_follows);
            eprintln!();
            prune_orphan_nodes(&mut lock);

            eprintln!();
            let node_hits = FlakeNodeVisits::count_from_index(&lock, lock.root_index());
            elog!(
                :bold (:bright_magenta "Flake input nodes' reference counts", :bright_green "after successful pruning" :bright_magenta ":");
                &node_hits
            );
            eprintln!();

            serialize_to_json_output(&lock, output, overwrite, pretty)
        }
        Command::Count {
            json,
            lock_file,
            output_opts:
                JsonOutputOptions {
                    in_place: _,
                    overwrite,
                    pretty,
                    output,
                },
        } => {
            let lock = read_flake_lock(lock_file);
            let node_hits = FlakeNodeVisits::count_from_index(&lock, lock.root_index());
            if json {
                serialize_to_json_output(&*node_hits, output, overwrite, pretty)
            } else {
                logln!(:bold :bright_magenta "Flake input nodes' reference counts:"; &node_hits)
            }
        }
    }
}

fn read_flake_lock(lock_file: Input) -> LockFile {
    let reader = lock_file
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

    lock
}

fn serialize_to_json_output(value: impl Serialize, output: Output, overwrite: bool, pretty: bool) {
    let writer = output
        .create(!overwrite)
        .unwrap_or_else(|e| panic!("Could not write to output: {e}"));

    let res = if pretty {
        value.serialize(&mut Serializer::pretty(writer))
    } else {
        value.serialize(&mut Serializer::new(writer))
    };

    if let Err(e) = res {
        panic!("Failed while serializing to output, file is probably corrupt: {e}")
    }
}

fn substitute_flake_inputs_with_follows(lock: &LockFile, indexed: bool) {
    elogln!(:bold :bright_magenta "Redirecting inputs to imitate follows behavior.");

    let root = lock.root().expect(EXPECT_ROOT_EXIST);
    for (input_name, input_index) in root
        .iter_edges()
        .filter_map(|(name, edge)| edge.index().map(|index| (name, index)))
    {
        elogln!(:bold (:bright_cyan "Replacing inputs for", :green "'{input_name}'"), :dimmed "(" :dimmed :italic "'{input_index}'" :dimmed ")");
        let input = &*lock
            .get_node(&*input_index)
            .expect("a node to exist with this index");
        substitute_node_inputs_with_root_inputs(lock, input, indexed);
    }
}

/// When `indexed == false`, the input replacements all will reference identically
/// named inputs from the root node. This imitates input following behavior.
///
/// Otherwise, if `indexed == true`, the each input replacement will be cloned
/// verbatim from the root node, most likely retaining a `NodeEdge::Indexed`.
fn substitute_node_inputs_with_root_inputs(lock: &LockFile, node: &Node, indexed: bool) {
    let root = lock.root().expect(EXPECT_ROOT_EXIST);
    for (edge_name, mut edge) in node.iter_edges_mut() {
        if let Some(root_edge) = root.get_edge(edge_name) {
            if indexed {
                let old = std::mem::replace(&mut *edge, (*root_edge).clone());
                elogln!("-", :yellow "'{edge_name}'", "now references", :italic :purple "'{edge}'", :dimmed "(was '{old}')");
            } else {
                let old = std::mem::replace(&mut *edge, NodeEdge::from_iter([edge_name]));
                elogln!("-", :yellow "'{edge_name}'", "now follows", :green "'{edge}'", :dimmed "(was '{old}')");
            }
        } else {
            elogln!(
                :bold (:cyan "No suitable replacement for", :yellow "'{edge_name}'"),
                :dimmed "(" :dimmed :italic ("'" (lock.resolve_edge(&edge).unwrap()) "'") :dimmed ")"
            );
        }
    }
}

fn prune_orphan_nodes(lock: &mut LockFile) {
    elogln!(:bold :bright_magenta "Pruning orphaned nodes from modified lock.");

    let node_hits = FlakeNodeVisits::count_from_index(lock, lock.root_index());

    let dead_nodes = node_hits
        .into_inner()
        .into_iter()
        .filter(|&(_, count)| count == 0)
        .map(|(index, _)| index.to_owned())
        .collect::<Vec<_>>();

    for index in dead_nodes {
        lock.remove_node(&index);
        elogln!("- removed", :red "'{index}'");
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
