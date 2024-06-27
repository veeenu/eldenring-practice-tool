use std::path::PathBuf;

use hudhook::tracing::info;
use pkg_version::*;
use semver::*;

use crate::util;

const PRACTICE_TOOL_VERSION: Version = Version {
    major: pkg_version_major!(),
    minor: pkg_version_minor!(),
    patch: pkg_version_patch!(),
    pre: Prerelease::EMPTY,
    build: BuildMetadata::EMPTY,
};

const UPDATE_URL: &str =
    "https://api.github.com/repos/veeenu/eldenring-practice-tool/releases/latest";

pub enum Update {
    Available { url: String, notes: String },
    UpToDate,
    Error(String),
}

impl Update {
    pub fn check() -> Self {
        info!("Checking for updates...");
        #[derive(serde::Deserialize)]
        struct GithubRelease {
            tag_name: String,
            html_url: String,
            body: String,
        }

        let release = match ureq::get(UPDATE_URL).call() {
            Ok(release) => release,
            Err(e) => return Update::Error(e.to_string()),
        };

        let release = match release.into_json::<GithubRelease>() {
            Ok(release) => release,
            Err(e) => return Update::Error(e.to_string()),
        };

        let version = match Version::parse(&release.tag_name) {
            Ok(version) => version,
            Err(e) => return Update::Error(e.to_string()),
        };

        if version > PRACTICE_TOOL_VERSION {
            let notes = match release.body.find("## What's Changed") {
                Some(i) => release.body[..i].trim().to_string(),
                None => release.body,
            };
            let notes = format!(
                "A new version of the practice tool is available!\n\nLatest version:    \
                 {}\nInstalled version: {}\n\nRelease notes:\n{}\n",
                version, PRACTICE_TOOL_VERSION, notes
            );

            let url = release.html_url;

            Update::Available { url, notes }
        } else {
            Update::UpToDate
        }
    }
}

#[derive(Clone)]
pub enum Install {
    Necessary { source: PathBuf, dest: PathBuf },
    Unnecessary,
    Error(String),
}

impl Install {
    pub fn check() -> Self {
        let dll_path = match util::get_dll_path() {
            Some(dll_path) => dll_path,
            None => return Install::Error("Could not determine path to tool dll".to_string()),
        };

        let exe = match std::env::current_exe() {
            Ok(exe) => exe,
            Err(e) => {
                return Install::Error(format!("Could not determine path to eldenring.exe: {e}"))
            },
        };

        let game_path = match exe.parent() {
            Some(game_path) => game_path,
            None => return Install::Error("Could not determine game directory".to_string()),
        };

        let dinput8_path = game_path.join("dinput8.dll");

        if dinput8_path.exists() {
            Self::Unnecessary
        } else {
            Self::Necessary { source: dll_path, dest: dinput8_path }
        }
    }

    pub fn install(&mut self) {
        *self = self.install_inner();
    }

    fn install_inner(&mut self) -> Self {
        if let Install::Necessary { source, dest } = self {
            if let Err(e) = std::fs::copy(&source, &dest) {
                return Install::Error(format!(
                    "Couldn't install practice tool DLL: {e}\nTried copying {source:?} to {dest:?}"
                ));
            }

            let source_conf = source.parent().map(|p| p.join("jdsd_er_practice_tool.toml"));
            let dest_conf = dest.parent().map(|p| p.join("jdsd_er_practice_tool.toml"));

            let (source_conf, dest_conf) = match (source_conf, dest_conf) {
                (Some(a), Some(b)) => (a.to_path_buf(), b.to_path_buf()),
                (source_conf, dest_conf) => {
                    return Install::Error(format!(
                        "Couldn't determine configuration paths:\n{source_conf:?} -> {dest_conf:?}"
                    ));
                },
            };

            if let Err(e) = std::fs::copy(&source_conf, &dest_conf) {
                return Install::Error(format!(
                    "Couldn't install practice tool configuration: {e}\nTried copying \
                     {source_conf:?} to {dest_conf:?}"
                ));
            }

            Install::Unnecessary
        } else {
            self.clone()
        }
    }
}
