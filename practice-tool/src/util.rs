use std::ffi::{OsStr, OsString};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::prelude::OsStringExt;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use hudhook::inject::Process;
use hudhook::tracing::*;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use windows::core::{PCSTR, *};
use windows::Win32::Foundation::{HMODULE, HWND, MAX_PATH};
use windows::Win32::System::LibraryLoader::{
    GetModuleFileNameW, GetModuleHandleExA, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};
use windows::Win32::System::Threading::{QueryFullProcessImageNameW, PROCESS_NAME_FORMAT};
use windows::Win32::UI::WindowsAndMessaging::*;

/// Create a Windows message box.
pub fn message_box<S: AsRef<str>, T: AsRef<str>>(
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

/// Return the path of the implementor's DLL.
pub fn get_dll_path() -> Option<PathBuf> {
    let mut hmodule: HMODULE = Default::default();
    // SAFETY
    // This is reckless, but it should never fail, and if it does, it's ok to crash
    // and burn.
    let gmh_result = unsafe {
        GetModuleHandleExA(
            GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT | GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
            PCSTR(c"DllMain".as_ptr() as *const u8),
            &mut hmodule,
        )
    };

    if let Err(e) = gmh_result {
        error!("get_dll_path: GetModuleHandleExA error: {e:?}");
        return None;
    }

    let mut sz_filename = [0u16; MAX_PATH as _];
    // SAFETY
    // pointer to sz_filename always defined and MAX_PATH bounds manually checked
    let len = unsafe { GetModuleFileNameW(hmodule, &mut sz_filename) } as usize;

    Some(OsString::from_wide(&sz_filename[..len]).into())
}

/// Retrieve the DLL path from the current executable's directory.
pub fn get_dll_path_exe() -> Result<PathBuf> {
    let mut dll_path = std::env::current_exe().unwrap();
    dll_path.pop();
    dll_path.push("jdsd_er_practice_tool.dll");

    if !dll_path.exists() {
        dll_path.pop();
        dll_path.push("libjdsd_er_practice_tool");
        dll_path.set_extension("dll");
    }

    Ok(dll_path.canonicalize()?)
}

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
