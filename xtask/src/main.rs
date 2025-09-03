use std::ffi::OsStr;
use std::{env, fs, iter};

use anyhow::{bail, Context, Result};
use practice_tool_tasks::{
    cargo_command, project_root, steam_command, target_path, Distribution, FileInstall,
};

mod codegen;

const APPID: u32 = 1245620;

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let task = env::args().nth(1);
    match task.as_deref() {
        Some("dist") => dist()?,
        Some("codegen") => codegen::codegen()?,
        Some("inject") => inject(env::args().skip(1))?,
        Some("run") => run()?,
        Some("install") => install()?,
        Some("uninstall") => uninstall()?,
        Some("help") => print_help(),
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        r#"
Tasks:

run ............. compile and start the practice tool
dist ............ build distribution artifacts
codegen ......... generate Rust code: parameters, base addresses, ...
inject <args> ... standalone dll inject
install ......... install standalone dll to $ER_PATH
uninstall ....... uninstall standalone dll from $ER_PATH
help ............ print this help
"#
    );
}

fn run() -> Result<()> {
    let status = cargo_command("build")
        .args(["--lib", "--package", "eldenring-practice-tool"])
        .status()
        .context("cargo")?;

    if !status.success() {
        bail!("cargo build failed");
    }

    fs::copy(
        project_root().join("jdsd_er_practice_tool.toml"),
        target_path("debug").join("jdsd_er_practice_tool.toml"),
    )?;

    let dll_path = target_path("debug").join("libjdsd_er_practice_tool.dll").canonicalize()?;

    inject(iter::once(dll_path))?;

    Ok(())
}

fn dist() -> Result<()> {
    Distribution::new("jdsd_er_practice_tool.zip")
        .with_artifact("libjdsd_er_practice_tool.dll", "jdsd_er_practice_tool.dll")
        .with_artifact("jdsd_er_practice_tool.exe", "jdsd_er_practice_tool.exe")
        .with_file("lib/data/RELEASE-README.txt", "README.txt")
        .with_file("jdsd_er_practice_tool.toml", "jdsd_er_practice_tool.toml")
        .build(&["--locked", "--release", "--workspace", "--exclude", "xtask"])
}

fn install() -> Result<()> {
    let status = cargo_command("build")
        .args(["--lib", "--release", "--package", "eldenring-practice-tool"])
        .status()
        .context("cargo")?;

    if !status.success() {
        bail!("cargo build failed");
    }

    FileInstall::new()
        .with_file(target_path("release").join("libjdsd_er_practice_tool.dll"), "dinput8.dll")
        .with_file(project_root().join("jdsd_er_practice_tool.toml"), "jdsd_er_practice_tool.toml")
        .install("ER_PATH")?;

    Ok(())
}

fn uninstall() -> Result<()> {
    FileInstall::new()
        .with_file(target_path("release").join("libjdsd_er_practice_tool.dll"), "dinput8.dll")
        .with_file(project_root().join("jdsd_er_practice_tool.toml"), "jdsd_er_practice_tool.toml")
        .uninstall("DSIII_PATH")?;

    Ok(())
}

fn inject<S: AsRef<OsStr>>(args: impl Iterator<Item = S>) -> Result<()> {
    cargo_command("build").args(["--release", "--bin", "inject"]).status().context("cargo")?;

    steam_command(target_path("release").join("inject"), APPID)?
        .args(args)
        .status()
        .context("inject")?;

    Ok(())
}
