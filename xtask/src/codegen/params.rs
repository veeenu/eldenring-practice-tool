use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::Command;

use regex::Regex;

use crate::{project_root, Result};

pub(crate) fn codegen() -> Result<()> {
    checkout_paramdex()?;
    run_python_script()?;
    codegen_param_names()?;

    Ok(())
}

fn run_python_script() -> Result<()> {
    let python = env::var("PYTHON").unwrap_or_else(|_| "python".to_string());
    let cmd = Command::new(python)
        .current_dir(project_root().join("target"))
        .args(&[
            project_root().join("xtask/src/codegen/params.py"),
            project_root().join("target/Paramdex"),
            project_root().join("xtask"),
        ])
        .output()
        .map_err(|e| format!("python: {}", e))?;

    if !cmd.status.success() {
        return Err("python codegen failed".into());
    }

    File::create(project_root().join("lib/libeldenring/src/codegen/param_data.rs"))?
        .write_all(&cmd.stdout)?;

    Ok(())
}

fn codegen_param_names() -> Result<()> {
    let mut data: HashMap<String, HashMap<usize, String>> = HashMap::new();

    let files_with_content = project_root()
        .join("target/Paramdex/ER/Names")
        .read_dir()?
        .flat_map(|entry| {
            entry.map(|entry| entry.path()).map(|path| {
                if path.is_file() && Some("txt") == path.extension().and_then(OsStr::to_str) {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .flatten();

    let r = Regex::new(r"^(\d+)\s+(.+)").unwrap();

    for path in files_with_content {
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();

        let data_contents: HashMap<_, _> = BufReader::new(File::open(path)?)
            .lines()
            .filter_map(|line| {
                let line = line.ok()?;
                let cap = r.captures(&line)?;

                let id: usize = cap[1].parse().ok()?;
                let name: String = cap[2].to_string();
                Some((id, name))
            })
            .collect();

        data.insert(stem, data_contents);
    }

    serde_json::to_writer(
        File::create(project_root().join("lib/libeldenring/src/codegen/param_names.json"))?,
        &data,
    )?;

    Ok(())
}

fn checkout_paramdex() -> Result<()> {
    let git = env::var("GIT").unwrap_or_else(|_| "git".to_string());

    if project_root().join("target/Paramdex").exists() {
        let status = Command::new(&git)
            .current_dir(project_root().join("target/Paramdex"))
            .args(["fetch"])
            .status()
            .map_err(|e| format!("git: {}", e))?;

        if !status.success() {
            return Err("git fetch failed".into());
        }

        let status = Command::new(&git)
            .current_dir(project_root().join("target/Paramdex"))
            .args(["pull"])
            .status()
            .map_err(|e| format!("git: {}", e))?;

        if !status.success() {
            return Err("git pull failed".into());
        }
    } else {
        let status = Command::new(&git)
            .current_dir(project_root().join("target"))
            .args(["clone", "https://github.com/soulsmods/Paramdex.git"])
            .status()
            .map_err(|e| format!("git: {}", e))?;

        if !status.success() {
            return Err("git clone failed".into());
        }
    }

    Ok(())
}
