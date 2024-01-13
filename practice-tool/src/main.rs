// johndisandonato's Elden Ring Practice Tool
// Copyright (C) 2022  johndisandonato <https://github.com/veeenu>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::prelude::AsRawHandle;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use dll_syringe::process::OwnedProcess;
use dll_syringe::Syringe;
use hudhook::tracing::{debug, trace};
use pkg_version::*;
use semver::*;
use textwrap_macros::dedent;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::Threading::{QueryFullProcessImageNameW, PROCESS_NAME_FORMAT};
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MB_YESNO, MESSAGEBOX_RESULT,
    MESSAGEBOX_STYLE,
};

fn message_box<S: AsRef<str>, T: AsRef<str>>(
    caption: S,
    text: T,
    style: MESSAGEBOX_STYLE,
) -> MESSAGEBOX_RESULT {
    let caption = OsStr::new("Elden Ring Practice Tool - ")
        .encode_wide()
        .chain(OsStr::new(caption.as_ref()).encode_wide())
        .chain(Some(0))
        .collect::<Vec<_>>();
    let text = OsStr::new(text.as_ref()).encode_wide().chain(Some(0)).collect::<Vec<_>>();

    unsafe { MessageBoxW(HWND(0), PCWSTR(text.as_ptr()), PCWSTR(caption.as_ptr()), style) }
}

fn err_to_string<T: std::fmt::Display>(e: T) -> String {
    format!("Error: {}", e)
}

fn get_current_version() -> Version {
    Version {
        major: pkg_version_major!(),
        minor: pkg_version_minor!(),
        patch: pkg_version_patch!(),
        pre: Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    }
}

fn get_latest_version() -> Result<(Version, String, String), String> {
    #[derive(serde::Deserialize)]
    struct GithubRelease {
        tag_name: String,
        html_url: String,
        body: String,
    }

    let release =
        ureq::get("https://api.github.com/repos/veeenu/eldenring-practice-tool/releases/latest")
            .call()
            .map_err(|e| format!("{}", e))?
            .into_json::<GithubRelease>()
            .map_err(|e| format!("{}", e))?;

    let version = Version::parse(&release.tag_name).map_err(err_to_string)?;

    Ok((version, release.html_url, release.body))
}

fn check_eac(handle: HANDLE) -> Result<bool> {
    let mut buf = [0u16; 256];
    let mut len = 256u32;
    let exe_path = PWSTR(buf.as_mut_ptr());
    unsafe { QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), exe_path, &mut len) }?;
    let exe_path = PathBuf::from(unsafe { exe_path.to_string()? });
    let exe_cwd = exe_path.parent().unwrap(); // Unwrap ok: must be in a Game directory anyway

    let steam_appid_path = exe_cwd.join("steam_appid.txt");
    debug!("{steam_appid_path:?} {}", steam_appid_path.exists());
    if !steam_appid_path.exists() {
        message_box(
            "EAC was not bypassed",
            "The EAC bypass is not applied.\n\nNo worries! I can apply that for you.",
            MB_ICONERROR,
        );
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(steam_appid_path)
            .context("Couldn't open steam_appid.txt")?;
        file.write_all(b"1245620").context("Couldn't write steam_appid.txt")?;

        message_box(
            "EAC bypassed",
            "EAC is now bypassed successfully!\n\nRestart the game and the practice \
             tool.\n\nRemember to double-click eldenring.exe.",
            MB_ICONINFORMATION,
        );

        return Ok(true);
    }

    Ok(false)
}

fn perform_injection() -> Result<()> {
    let mut dll_path = std::env::current_exe().unwrap();
    dll_path.pop();
    dll_path.push("jdsd_er_practice_tool.dll");

    if !dll_path.exists() {
        dll_path.pop();
        dll_path.push("libjdsd_er_practice_tool");
        dll_path.set_extension("dll");
    }

    let dll_path = dll_path.canonicalize()?;
    trace!("Injecting {:?}", dll_path);

    let process = OwnedProcess::find_first_by_name("eldenring.exe").ok_or_else(|| {
        anyhow!(dedent!(
            r#"
            Could not find the ELDEN RING process.

            Make sure to follow these steps
            - Disable running antivirus software and uninstall any mods.
            - Start Steam (offline mode is fine).
            - Double-click eldenring.exe
              (Steam > ELDEN RING > Manage > Browse Local Files).
            - Double-click jdsd_er_practice_tool.exe.
            "#
        )
        .trim())
    })?;

    trace!("Checking EAC...");
    if check_eac(HANDLE(process.as_raw_handle() as _))? {
        return Ok(());
    }

    let syringe = Syringe::for_process(process);
    syringe.inject(dll_path).map_err(|e| {
        anyhow!(
            "Could not hook the practice tool: {e}.\n\nPlease make sure you have no antiviruses \
             running, EAC is properly bypassed, and you are running an unmodded and legitimate \
             version of the game."
        )
    })?;

    Ok(())
}

fn main() -> Result<()> {
    {
        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_thread_names(true)
            .with_ansi(true)
            .boxed();

        tracing_subscriber::registry().with(LevelFilter::TRACE).with(stdout_layer).init();
    }

    let current_version = get_current_version();

    match get_latest_version() {
        Ok((latest_version, download_url, release_notes)) => {
            if latest_version > current_version {
                let release_notes = match release_notes.find("## What's Changed") {
                    Some(i) => release_notes[..i].trim(),
                    None => &release_notes,
                };
                let update_msg = format!(
                    "A new version of the practice tool is available!\n\nLatest version: \
                     {}\nInstalled version: {}\n\nRelease notes:\n{}\n\nDo you want to download \
                     the update?",
                    latest_version, current_version, release_notes
                );

                let msgbox_response =
                    message_box("Update available", update_msg, MB_YESNO | MB_ICONINFORMATION);

                if msgbox_response == IDYES {
                    open::that(download_url).ok();
                    return Ok(());
                }
            }
        },
        Err(e) => {
            message_box(
                "Error",
                format!("Could not check for a new version: {e}"),
                MB_OK | MB_ICONERROR,
            );
        },
    }

    if let Err(e) = perform_injection() {
        message_box("Error", e.to_string(), MB_OK | MB_ICONERROR);
    }

    Ok(())
}
