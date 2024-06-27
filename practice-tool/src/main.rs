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
use std::path::PathBuf;
use std::ptr::null_mut;

use anyhow::{anyhow, bail, Context, Result};
use hudhook::inject::Process;
use hudhook::tracing::debug;
use textwrap_macros::dedent;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use windows::core::*;
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::Threading::{
    QueryFullProcessImageNameW, TerminateProcess, PROCESS_NAME_FORMAT,
};
use windows::Win32::UI::Controls::{
    TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOG_BUTTON, TASKDIALOG_COMMON_BUTTON_FLAGS,
    TASKDIALOG_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::*;

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

fn get_game_directory(handle: HANDLE) -> Result<PathBuf> {
    let mut buf = [0u16; 256];
    let mut len = 256u32;
    let exe_path = PWSTR(buf.as_mut_ptr());
    unsafe { QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), exe_path, &mut len) }?;
    let exe_path = PathBuf::from(unsafe { exe_path.to_string()? });
    exe_path
        .parent()
        .ok_or_else(|| anyhow!("Couldn't find executable's parent directory"))
        .map(|p| p.to_path_buf())
}

fn check_eac(handle: HANDLE) -> Result<bool> {
    let exe_cwd = get_game_directory(handle)?;

    let steam_appid_path = exe_cwd.join("steam_appid.txt");
    debug!("Steam AppID path: {steam_appid_path:?} exists? {}", steam_appid_path.exists());
    if !steam_appid_path.exists() {
        message_box(
            "EAC was not bypassed",
            "The EAC bypass is not applied.\n\nNo worries! I can apply that for you.",
            MB_ICONERROR,
        );
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
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
    debug!("Injecting {:?}", dll_path);

    let process = Process::by_name("eldenring.exe").map_err(|_| {
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

    debug!("Checking EAC...");
    if check_eac(process.handle())? {
        return Ok(());
    }

    process.inject(dll_path).map_err(|e| {
        anyhow!(
            "Could not hook the practice tool: {e}.\n\nPlease make sure you have no antiviruses \
             running, EAC is properly bypassed, and you are running an unmodded and legitimate \
             version of the game."
        )
    })?;

    Ok(())
}

fn install() -> Result<()> {
    let mut dll_path = std::env::current_exe().unwrap();
    dll_path.pop();
    dll_path.push("jdsd_er_practice_tool.dll");

    if !dll_path.exists() {
        dll_path.pop();
        dll_path.push("libjdsd_er_practice_tool");
        dll_path.set_extension("dll");
    }

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

    let process = Process::by_name("eldenring.exe").map_err(|_| {
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

    let game_install_path = get_game_directory(process.handle())?;

    debug!("{:?}", game_install_path);

    debug!("Checking EAC...");
    if check_eac(process.handle())? {
        return Ok(());
    }

    debug!("Installing...");
    let dll_path_dest = game_install_path.join("dinput8.dll");
    let config_path_dest = game_install_path.join("jdsd_er_practice_tool.toml");

    if dll_path_dest.exists()
        && message_box(
            "Close Elden Ring",
            dedent!(
                r#"
                It appears that the tool is already installed in the
                currently running Elden Ring process.

                I will close it to continue the installation. Make sure 
                you have quit out to main menu, and then click "Ok". 
                If you don't want to close the game right now, click 
                "Cancel" to abort the tool's installation.
                "#
            )
            .trim(),
            MB_OKCANCEL | MB_ICONINFORMATION,
        ) != MESSAGEBOX_RESULT(1)
    {
        debug!("Aborting installation");
        return Ok(());
    }

    unsafe { TerminateProcess(process.handle(), 1) }
        .map_err(|e| anyhow!("Could not close Elden Ring: {e}"))?;

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

    message_box("Success", "The tool was installed successfully.", MB_ICONINFORMATION);

    Ok(())
}

enum Action {
    Run,
    Install,
    Uninstall,
}

fn action() -> Result<Action> {
    let buttons = [
        TASKDIALOG_BUTTON { nButtonID: 100, pszButtonText: w!("Run the tool") },
        TASKDIALOG_BUTTON { nButtonID: 101, pszButtonText: w!("Install/update the tool") },
        TASKDIALOG_BUTTON { nButtonID: 102, pszButtonText: w!("Uninstall the tool") },
    ];

    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        hwndParent: HWND_DESKTOP,
        dwFlags: TASKDIALOG_FLAGS(0),
        dwCommonButtons: TASKDIALOG_COMMON_BUTTON_FLAGS(0),
        pszWindowTitle: w!("johndisandonato's Elden Ring practice tool"),
        pszMainInstruction: w!("What do you want to do?"),
        pszContent: PCWSTR(null_mut()),
        cButtons: buttons.len() as u32,
        pButtons: buttons.as_ptr(),
        nDefaultButton: 100,
        ..Default::default()
    };

    let mut button_pressed = 0;

    unsafe {
        TaskDialogIndirect(&config, Some(&mut button_pressed), None, None)?;
    }

    match button_pressed {
        100 => Ok(Action::Run),
        101 => Ok(Action::Install),
        102 => Ok(Action::Uninstall),
        _ => Err(anyhow!("Error")),
    }
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

        tracing_subscriber::registry().with(LevelFilter::DEBUG).with(stdout_layer).init();
    }

    match action()? {
        Action::Run => {
            if let Err(e) = perform_injection() {
                message_box("Error", e.to_string(), MB_OK | MB_ICONERROR);
            }
        },
        Action::Install => {
            if let Err(e) = install() {
                message_box("Error", e.to_string(), MB_OK | MB_ICONERROR);
            }
        },
        Action::Uninstall => {
            message_box(
                "Uninstall",
                "Uninstall has not yet been implemented. Coming Soon (TM)!\n\nOpen Elden Ring's \
                 installation directory and remove dinput8.dll and jdsd_er_practice_tool.toml \
                 manually.",
                MB_OK | MB_ICONERROR,
            );
        },
    }

    Ok(())
}
