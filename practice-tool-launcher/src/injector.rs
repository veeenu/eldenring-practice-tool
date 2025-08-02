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

use anyhow::{Result, anyhow};
use hudhook::inject::Process;
use hudhook::tracing::debug;
use libjdsd_er_practice_tool::update::Update;
use libjdsd_er_practice_tool::util::*;
use textwrap_macros::dedent;
use windows::Win32::UI::WindowsAndMessaging::*;

fn perform_injection() -> Result<()> {
    debug!("Looking for ELDEN RING process...");
    let process = Process::by_name("eldenring.exe").map_err(|_| {
        anyhow!(dedent!(
            r#"
            Could not find the ELDEN RING process.

            If it is not running, start it.
            
            If you have and it's not working, make sure to follow these steps:
            - Disable running antivirus software, uninstall any mods and stop overlay programs (RTSS/FRAPS).
            - Start Steam (offline mode is fine).
            - Double-click eldenring.exe
              (Steam > ELDEN RING > Manage > Browse Local Files).
            - Double-click jdsd_er_practice_tool.exe.
            "#
        )
        .trim())
    })?;

    debug!("Searching for tool DLL...");
    let dll_path = get_dll_path_exe().map_err(|e| {
        anyhow!(
            "Could not find the tool DLL: {e}.\n\nPlease make sure you have extracted the \
             practice tool's zip file contents to a directory before trying again."
        )
    })?;

    debug!("Checking EAC...");
    if check_eac(&process)? {
        return Ok(());
    }

    debug!("Injecting {:?}...", dll_path);
    process.inject(dll_path).map_err(|e| {
        anyhow!(
            "Could not hook the practice tool: {e}.\n\nPlease make sure you have no antiviruses \
             running, EAC is properly bypassed, no other overlay tools like FRAPS and RTSS are \
             running, and you are running an unmodded and legitimate version of the game."
        )
    })?;

    Ok(())
}

pub fn run() -> Result<()> {
    tracing_init();

    match Update::check() {
        Update::Error(e) => {
            let _ = message_box(
                "Elden Ring Practice Tool - Error",
                format!("Could not check for updates: {e}"),
                MB_OK | MB_ICONERROR,
            );
        },
        Update::Available { url, notes } => {
            if let MESSAGEBOX_RESULT(1) = message_box(
                "Elden Ring Practice Tool - Update available",
                format!("{notes}\n\nDo you want to download it?"),
                MB_OKCANCEL | MB_ICONINFORMATION,
            ) {
                return Ok(open::that(&url)?);
            }
        },
        Update::UpToDate => {},
    }

    if let Err(e) = perform_injection() {
        message_box("Error", e.to_string(), MB_OK | MB_ICONERROR);
    }

    Ok(())
}
