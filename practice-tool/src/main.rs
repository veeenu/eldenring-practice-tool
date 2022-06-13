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

use dll_syringe::process::OwnedProcess;
use dll_syringe::Syringe;
use pkg_version::*;
use semver::*;
use simplelog::*;
use windows::core::PCSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxA, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MB_YESNO,
};
// use winapi::shared::windef::*;
// use winapi::um::winuser::*;

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

fn perform_injection() -> Result<(), String> {
    let mut dll_path = std::env::current_exe().unwrap();
    dll_path.pop();
    dll_path.push("jdsd_er_practice_tool.dll");

    if !dll_path.exists() {
        dll_path.pop();
        dll_path.push("libjdsd_er_practice_tool");
        dll_path.set_extension("dll");
    }

    let dll_path = dll_path.canonicalize().map_err(err_to_string)?;
    log::trace!("Injecting {:?}", dll_path);

    let process = OwnedProcess::find_first_by_name("eldenring.exe")
        .ok_or_else(|| "Could not find process".to_string())?;
    let syringe = Syringe::for_process(process);
    syringe.inject(dll_path).map_err(|e| format!("{e}"))?;

    Ok(())
}

fn main() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Trace,
        ConfigBuilder::new()
            .add_filter_allow("jdsd_er_practice_tool".to_string())
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .ok();
    let current_version = get_current_version();

    match get_latest_version() {
        Ok((latest_version, download_url, release_notes)) => {
            if latest_version > current_version {
                let update_msg = format!(
          "A new version of the practice tool is available!\n\nLatest version: {}\nInstalled version: {}\n\nRelease notes:\n{}\n\nDo you want to download the update?\0",
          latest_version, current_version, release_notes
        );

                let msgbox_response = unsafe {
                    MessageBoxA(
                        HWND(0),
                        PCSTR(update_msg.as_str().as_ptr()),
                        PCSTR("Update available\0".as_ptr()),
                        MB_YESNO | MB_ICONINFORMATION,
                    )
                };

                if IDYES == msgbox_response {
                    open::that(download_url).ok();
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Could not check for a new version: {}\0", e);
            unsafe {
                MessageBoxA(
                    HWND(0),
                    PCSTR(error_msg.as_str().as_ptr()),
                    PCSTR("Error\0".as_ptr()),
                    MB_OK | MB_ICONERROR,
                );
            }
        }
    }

    if let Err(e) = perform_injection() {
        let error_msg = format!("{}\0", e);
        unsafe {
            MessageBoxA(
                HWND(0),
                PCSTR(error_msg.as_str().as_ptr()),
                PCSTR("Error\0".as_ptr()),
                MB_OK | MB_ICONERROR,
            );
        }
    }
}
