use std::ptr::null_mut;

use log::*;
use once_cell::sync::Lazy;
use widestring::U16CString;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};

pub use crate::prelude::base_addresses::Version;

pub static VERSION: Lazy<Version> = Lazy::new(get_version);

fn get_version() -> Version {
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
    Version::from((major, minor, patch))
}
