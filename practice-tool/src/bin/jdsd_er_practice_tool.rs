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

use anyhow::{anyhow, bail, Result};
use hudhook::inject::Process;
use hudhook::tracing::{debug, error};
use libjdsd_er_practice_tool::util::*;
use libjdsd_er_practice_tool::{check_update, events};
use practice_tool_core::update::Update;
use practice_tool_core_windows::message_box;
use practice_tool_core_windows::startup::{is_running, request_start};
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

    let events = events();
    if is_running(&events) {
        bail!(
            "The practice tool is already running.\n\nTo start a different copy, restart the game \
             first."
        );
    }

    debug!("Searching for tool DLL...");
    let mut dll_path = std::env::current_exe()?;
    dll_path.set_file_name("dinput8.dll");

    if !dll_path.exists() {
        bail!(
            "Could not find {}.\n\nExtract all the files from the zip archive before running the \
             practice tool, and make sure your antivirus didn't delete it.",
            dll_path.display()
        );
    }

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

    // A freshly loaded DLL is running by now. If it isn't, the game had already
    // loaded this very file at startup, so injecting it did nothing: ask
    // that copy to start instead.
    if !is_running(&events) {
        request_start(&events).map_err(|e| anyhow!("Could not start the practice tool: {e}."))?;
    }

    Ok(())
}

fn main() -> Result<()> {
    tracing_init();

    match check_update() {
        // Shown in the overlay.
        Update::Error(e) => error!("Could not check for updates: {e}"),
        Update::Available { url, notes } => {
            if let MESSAGEBOX_RESULT(1) = message_box(
                "Elden Ring Practice Tool - Update available",
                &format!("{notes}\n\nDo you want to download it?"),
                MB_OKCANCEL | MB_ICONINFORMATION,
            ) {
                return Ok(open::that(&url)?);
            }
        },
        Update::UpToDate => {},
    }

    if let Err(e) = perform_injection() {
        message_box("Elden Ring Practice Tool - Error", &e.to_string(), MB_OK | MB_ICONERROR);
    }

    Ok(())
}
