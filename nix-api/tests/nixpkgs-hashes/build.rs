use std::ffi::OsString;
use std::process::Stdio;

use smol::io::{AsyncBufReadExt, BufReader};
use smol::process::Command;
use smol::stream::StreamExt;
use sonic_rs::JsonValueTrait;

fn main() -> std::io::Result<()> {
    println!("cargo::rerun-if-changed=npins/sources.json");

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

        while let Some(json_line) = json_lines.try_next().await? {
            let Ok(drv_path) = sonic_rs::get_from_str(&json_line, ["drvPath"]) else {
                assert!(sonic_rs::get_from_str(&json_line, ["error"]).is_ok());
                continue;
            };
            let drv_path = drv_path.as_str().unwrap();
            println!("drv_path = {drv_path}")
        }

        let status = eval_drvs.status().await?;
        if !status.success() {
            println!("cargo::warning=nix-eval-jobs exited with {status}");
        }

        Ok(())
    })
}
