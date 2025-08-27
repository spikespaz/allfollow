use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::process::Stdio;
use std::sync::LazyLock;

use smol::io::{AsyncBufReadExt, BufReader};
use smol::process::Command;
use smol::stream::StreamExt;
use sonic_rs::{JsonValueTrait, LazyValue, PointerTree};

#[derive(Debug, PartialEq, Eq, Hash)]
struct Hash {
    pub hash: String,
    pub algo: Option<String>,
}

struct DerivationHashes {
    pub env: Option<Hash>,
    pub outputs: Vec<(String, Hash)>,
}

fn main() -> std::io::Result<()> {
    println!("cargo::rerun-if-changed=npins/sources.json");
    println!("cargo::rerun-if-changed=nixpkgs-release.nix");

    if cfg!(rust_analyzer) {
        println!("cargo::warning=skipping nix-eval-jobs when invoked from rust-analyzer");
        return Ok(());
    }

    smol::block_on(async {
        let mut eval_drvs = Command::new("nix-eval-jobs")
            .arg("--workers")
            .arg(std::env::var_os("NUM_JOBS").unwrap())
            .arg("--force-recurse")
            .arg("--expr")
            .arg({
                let mut expr = OsString::new();
                expr.push("import ");
                expr.push(std::fs::canonicalize("./nixpkgs-release.nix").unwrap());
                expr
            })
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let eval_stdout = eval_drvs.stdout.take().unwrap();
        let mut json_lines = BufReader::new(eval_stdout).lines();

        let mut unique_hashes = HashSet::new();

        while let Some(json_line) = json_lines.try_next().await? {
            let Ok(drv_path) = sonic_rs::get_from_str(&json_line, ["drvPath"]) else {
                assert!(sonic_rs::get_from_str(&json_line, ["error"]).is_ok());
                continue;
            };
            let drv_path = drv_path.as_str().unwrap();

            let drv_hashes = collect_hashes_for_many_derivations(&[drv_path]).await?;
            for (drv_path, DerivationHashes { env, outputs }) in drv_hashes {
                if let Some(env_hash) = env {
                    println!("{drv_path} = {env_hash:?}");
                    unique_hashes.insert(env_hash);
                }
                for (out_name, out_hash) in outputs {
                    println!("{drv_path}/{out_name} {out_hash:?}");
                    unique_hashes.insert(out_hash);
                }
            }
        }

        let status = eval_drvs.status().await?;
        if !status.success() {
            println!("cargo::warning=nix-eval-jobs exited with {status}");
        }

        Ok(())
    })
}

async fn collect_hashes_for_many_derivations(
    drvs: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> std::io::Result<Vec<(String, DerivationHashes)>> {
    let output = Command::new("nix")
        .args(["derivation", "show", "--recursive"])
        .args(drvs)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await?;
    if !output.status.success() {
        todo!()
    }
    let drv_hashes = sonic_rs::to_object_iter(output.stdout.as_slice()).map(|res| {
        let (drv_path, drv_json) = res.unwrap();
        (
            drv_path.to_string(),
            collect_hashes_for_derivation(&drv_json),
        )
    });
    Ok(drv_hashes.collect())
}

fn collect_hashes_for_derivation(json: &LazyValue) -> DerivationHashes {
    static PATHS: LazyLock<PointerTree> = LazyLock::new(|| {
        let mut paths = PointerTree::new();
        paths.add_path(&["env", "outputHash"]);
        paths.add_path(&["env", "outputHashAlgo"]);
        paths.add_path(&["outputs"]);
        paths
    });
    let values = sonic_rs::get_many(json.as_raw_str(), &PATHS).unwrap();
    let [env_hash, env_hash_algo, outputs] = values.try_into().unwrap();

    let env_hash = env_hash.as_ref().map(|v| v.as_str().unwrap());
    let env_hash_algo = env_hash_algo.as_ref().and_then(|v| match v {
        v if v.is_null() => None,
        v => Some(v.as_str().unwrap()),
    });
    let outputs = outputs.and_then(LazyValue::into_object_iter).unwrap();

    let env = env_hash.map(|hash| Hash {
        hash: hash.to_string(),
        algo: env_hash_algo.map(str::to_string),
    });

    let outputs = outputs
        .map(Result::unwrap)
        .filter_map(|(out_name, out_json)| {
            static PATHS: LazyLock<PointerTree> = LazyLock::new(|| {
                let mut paths = PointerTree::new();
                paths.add_path(&["hash"]);
                paths.add_path(&["hashAlgo"]);
                paths
            });
            let values = sonic_rs::get_many(out_json.as_raw_str(), &PATHS).unwrap();
            let [hash, algo] = values.try_into().unwrap();

            let hash = hash.map(|v| v.as_str().unwrap().to_string())?;
            let algo = algo.map(|v| v.as_str().unwrap().to_string()).unwrap();

            Some((out_name.to_string(), Hash::with_algo(hash, algo)))
        })
        .collect();

    DerivationHashes { env, outputs }
}

impl Hash {
    fn with_algo(hash: impl Into<String>, algo: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
            algo: Some(algo.into()),
        }
    }
}
