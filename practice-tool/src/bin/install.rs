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
use hudhook::tracing::debug;
use libjdsd_er_practice_tool::util::*;
use textwrap_macros::dedent;
use windows::Win32::System::Threading::{TerminateProcess, WaitForSingleObjectEx};
use windows::Win32::UI::WindowsAndMessaging::*;

fn install() -> Result<()> {
    message_box(
        "johndisandonato's Elden Ring practice tool",
        "Welcome to the Elden Ring practice tool's installer!\n\nPlease start Elden Ring, if you \
         haven't done so yet.\n\nIf you have multiple Elden Ring installations (e.g. different \
         patches for speedrunning), you have to run this installer for each of them.\n\nThe \
         installer will create a file named `dinput8.dll` next to the game's executable. If you \
         want to uninstall the tool, just remove it.\n\nLet's go!"
            .trim(),
        MB_OK,
    );

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
            - Double-click install.exe.
            "#
        )
        .trim())
    })?;

    debug!("Searching for tool DLL...");
    let dll_path = get_dll_path_exe()?;

    let config_path = dll_path
        .parent()
        .ok_or_else(|| anyhow!("{dll_path:?} has no parent"))?
        .join("jdsd_er_practice_tool.toml");

    if !config_path.exists() {
        bail!(
            "Could not find jdsd_er_practice_tool.toml.\nPlease make sure you have extracted the \
             zip file before running the installer."
        );
    }

    let game_install_path = get_game_directory(&process)?;

    debug!("Checking EAC...");
    if check_eac(&process)? {
        return Ok(());
    }

    debug!("Installing...");
    let dll_path_dest = game_install_path.join("dinput8.dll");
    let config_path_dest = game_install_path.join("jdsd_er_practice_tool.toml");

    if dll_path_dest.exists() {
        if message_box(
            "Close Elden Ring",
            "It appears that the tool is already installed in the currently running Elden Ring \
             process.\n\nI will close it to continue the installation. Make sure you have quit \
             out to main menu, and then click \"Ok\".\n\nIf you don't want to close the game \
             right now, click \"Cancel\" to abort the \ntool's installation.",
            MB_OKCANCEL | MB_ICONINFORMATION,
        ) != MESSAGEBOX_RESULT(1)
        {
            debug!("Aborting installation");
            return Ok(());
        } else {
            unsafe { TerminateProcess(process.handle(), 1) }
                .map_err(|e| anyhow!("Could not close Elden Ring: {e}"))?;
            unsafe { WaitForSingleObjectEx(process.handle(), 20000, false) };
        }
    }

    std::fs::copy(&dll_path, &dll_path_dest).map_err(|e| {
        anyhow!(
            "Could not install DLL: {e}\nWhile trying to copy\n{dll_path:?}\nto\n{dll_path_dest:?}"
        )
    })?;
    std::fs::copy(&config_path, &config_path_dest).map_err(|e| {
        anyhow!(
            "Could not install config file: {e}\nWhile trying to \
             copy\n{config_path:?}\nto\n{config_path_dest:?}"
        )
    })?;

    message_box(
        "Success",
        "The tool was installed successfully.\n\nTo use it, restart the game, and hold right \
         shift for a few seconds during startup until the tool appears on screen.\n\nHappy video \
         gaming!",
        MB_ICONINFORMATION,
    );

    Ok(())
}

fn main() -> Result<()> {
    tracing_init();

    if let Err(e) = install() {
        message_box("Error", e.to_string(), MB_OK | MB_ICONERROR);
    }

    Ok(())
}
