use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use hudhook::inject::Process;
use hudhook::tracing::*;
use practice_tool_core_windows::message_box;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use windows::core::*;
use windows::Win32::System::Threading::{QueryFullProcessImageNameW, PROCESS_NAME_FORMAT};
use windows::Win32::UI::WindowsAndMessaging::*;

/// Retrieve the game's directory from an Elden Ring process.
pub fn get_game_directory(process: &Process) -> Result<PathBuf> {
    let handle = process.handle();

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

/// Check whether EAC is enabled.
pub fn check_eac(process: &Process) -> Result<bool> {
    let exe_cwd = get_game_directory(process)?;

    let steam_appid_path = exe_cwd.join("steam_appid.txt");
    debug!("Steam AppID path: {steam_appid_path:?} exists? {}", steam_appid_path.exists());
    if !steam_appid_path.exists() {
        message_box(
            "Elden Ring Practice Tool - EAC was not bypassed",
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
            "Elden Ring Practice Tool - EAC bypassed",
            "EAC is now bypassed successfully!\n\nRestart the game and the practice \
             tool.\n\nRemember to double-click eldenring.exe.",
            MB_ICONINFORMATION,
        );

        return Ok(true);
    }

    Ok(false)
}

pub fn tracing_init() {
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_ansi(true)
        .boxed();

    tracing_subscriber::registry().with(LevelFilter::DEBUG).with(stdout_layer).init();
}
