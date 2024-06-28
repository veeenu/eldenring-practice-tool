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

use anyhow::{anyhow, Result};
use hudhook::inject::Process;
use hudhook::tracing::debug;
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
            - Disable running antivirus software and uninstall any mods.
            - Start Steam (offline mode is fine).
            - Double-click eldenring.exe
              (Steam > ELDEN RING > Manage > Browse Local Files).
            - Double-click jdsd_er_practice_tool.exe.
            "#
        )
        .trim())
    })?;

    debug!("Searching for tool DLL...");
    let dll_path = get_dll_path_exe()?;

    debug!("Checking EAC...");
    if check_eac(&process)? {
        return Ok(());
    }

    debug!("Injecting {:?}...", dll_path);
    process.inject(dll_path).map_err(|e| {
        anyhow!(
            "Could not hook the practice tool: {e}.\n\nPlease make sure you have no antiviruses \
             running, EAC is properly bypassed, and you are running an unmodded and legitimate \
             version of the game."
        )
    })?;

    Ok(())
}

fn main() -> Result<()> {
    tracing_init();

    if let Err(e) = perform_injection() {
        message_box("Error", e.to_string(), MB_OK | MB_ICONERROR);
    }

    Ok(())
}
