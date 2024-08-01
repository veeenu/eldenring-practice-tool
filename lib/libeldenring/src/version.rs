use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::sync::OnceLock;

use log::*;
use widestring::U16CString;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, MAX_PATH};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

pub use crate::prelude::base_addresses::Version;

static VERSION: OnceLock<Version> = OnceLock::new();

/// Ensures that the VERSION static gets filled, or returns an error.
/// The caller MUST exit cleanly in case of an error.
pub fn check_version() -> Result<Version, (u32, u32, u32)> {
    if let Some(version) = VERSION.get().copied() {
        return Ok(version);
    }

    let file_path = {
        let mut buf = vec![0u16; MAX_PATH as usize];
        unsafe { GetModuleFileNameW(GetModuleHandleW(None).unwrap(), &mut buf) };
        U16CString::from_vec_truncate(buf)
    };

    let mut version_info_size =
        unsafe { GetFileVersionInfoSizeW(PCWSTR(file_path.as_ptr()), None) };
    let mut version_info_buf = vec![0u8; version_info_size as usize];
    unsafe {
        GetFileVersionInfoW(
            PCWSTR(file_path.as_ptr()),
            0,
            version_info_size,
            version_info_buf.as_mut_ptr() as _,
        )
        .unwrap()
    };

    let mut version_info: *mut VS_FIXEDFILEINFO = null_mut();
    unsafe {
        VerQueryValueW(
            version_info_buf.as_ptr() as _,
            w!("\\\\\0"),
            &mut version_info as *mut *mut _ as _,
            &mut version_info_size,
        )
    };
    let version_info = unsafe { version_info.as_ref().unwrap() };
    let major = (version_info.dwFileVersionMS >> 16) & 0xffff;
    let minor = (version_info.dwFileVersionMS) & 0xffff;
    let patch = (version_info.dwFileVersionLS >> 16) & 0xffff;

    info!("Version {} {} {}", major, minor, patch);
    match Version::try_from((major, minor, patch)) {
        Ok(version) => {
            while VERSION.set(version).is_err() {}
            Ok(version)
        },
        Err(()) => {
            error_messagebox((major, minor, patch));
            Err((major, minor, patch))
        },
    }
}

pub fn get_version() -> Version {
    VERSION.get().copied().expect("Game version not found")
}

fn error_messagebox((major, minor, patch): (u32, u32, u32)) {
    let caption = OsStr::new("Elden Ring Practice Tool - Unsupported version")
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let text = OsStr::new(&format!(
        "The current game version, {major}.{minor}.{patch}, is not supported yet.\n\nAn update \
         will be released soon, please stay tuned!"
    ))
    .encode_wide()
    .chain(Some(0))
    .collect::<Vec<_>>();

    unsafe {
        MessageBoxW(HWND(0), PCWSTR(text.as_ptr()), PCWSTR(caption.as_ptr()), MB_OK | MB_ICONERROR)
    };
}
