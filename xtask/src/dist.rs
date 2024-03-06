use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::{cargo_command, project_root, target_path, Result};

pub(crate) fn dist() -> Result<()> {
    let status = cargo_command("build")
        .env("CARGO_XTASK_DIST", "true")
        .args(["--release", "--package", "eldenring-practice-tool"])
        .status()
        .map_err(|e| format!("cargo: {}", e))?;

    if !status.success() {
        return Err("cargo build failed".into());
    }

    let status = cargo_command("build")
        .env("CARGO_XTASK_DIST", "true")
        .args(["--release", "--package", "no-logo"])
        .status()
        .map_err(|e| format!("cargo: {}", e))?;

    if !status.success() {
        return Err("cargo build failed".into());
    }

    fs::remove_dir_all(dist_dir()).ok();
    fs::create_dir_all(dist_dir())?;

    // Create distribution zip file(s)

    struct DistZipFile {
        zip: ZipWriter<File>,
        file_options: FileOptions,
    }

    impl DistZipFile {
        fn new(zip_name: &str) -> Result<Self> {
            let zip = ZipWriter::new(File::create(dist_dir().join(zip_name))?);
            let file_options =
                FileOptions::default().compression_method(CompressionMethod::Deflated);

            Ok(Self { zip, file_options })
        }

        fn add(&mut self, src: PathBuf, dst: &str) -> Result<()> {
            self.add_map(src, dst, |buf| buf)
        }

        fn add_map<F>(&mut self, src: PathBuf, dst: &str, f: F) -> Result<()>
        where
            F: Fn(Vec<u8>) -> Vec<u8>,
        {
            let mut buf = Vec::new();
            File::open(src)
                .map_err(|e| format!("{}: Couldn't open file: {}", dst, e))?
                .read_to_end(&mut buf)
                .map_err(|e| format!("{}: Couldn't read file: {}", dst, e))?;

            let buf = f(buf);

            self.zip
                .start_file(dst, self.file_options)
                .map_err(|e| format!("{}: Couldn't start zip file: {}", dst, e))?;
            self.zip.write_all(&buf).map_err(|e| format!("{}: Couldn't write zip: {}", dst, e))?;
            Ok(())
        }
    }

    let mut dist = DistZipFile::new("jdsd_er_practice_tool.zip")?;

    dist.add(
        target_path("release").join("jdsd_er_practice_tool.exe"),
        "jdsd_er_practice_tool.exe",
    )?;
    dist.add(
        target_path("release").join("libjdsd_er_practice_tool.dll"),
        "jdsd_er_practice_tool.dll",
    )?;
    dist.add(target_path("release").join("dinput8.dll"), "dinput8.dll")?;
    dist.add(project_root().join("lib/data/RELEASE-README.txt"), "README.txt")?;
    dist.add(project_root().join("jdsd_er_practice_tool.toml"), "jdsd_er_practice_tool.toml")?;

    Ok(())
}

fn dist_dir() -> PathBuf {
    project_root().join("target/dist")
}
