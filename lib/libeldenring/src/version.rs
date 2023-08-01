use std::ptr::null_mut;
use std::sync::LazyLock;

use log::*;
use widestring::U16CString;
use windows::core::PCWSTR;
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};

pub use crate::prelude::base_addresses::Version;

pub static VERSION: LazyLock<Version> = LazyLock::new(get_version);

impl Version {
    pub fn tuple(&self) -> (u8, u8, u8) {
        match self {
            Version::V1_02_0 => (1, 2, 0),
            Version::V1_02_1 => (1, 2, 1),
            Version::V1_02_2 => (1, 2, 2),
            Version::V1_02_3 => (1, 2, 3),
            Version::V1_03_0 => (1, 3, 0),
            Version::V1_03_1 => (1, 3, 1),
            Version::V1_03_2 => (1, 3, 2),
            Version::V1_04_0 => (1, 4, 0),
            Version::V1_04_1 => (1, 4, 1),
            Version::V1_05_0 => (1, 5, 0),
            Version::V1_06_0 => (1, 6, 0),
            Version::V1_07_0 => (1, 7, 0),
            Version::V1_08_0 => (1, 8, 0),
            Version::V1_08_1 => (1, 8, 1),
            Version::V1_09_0 => (1, 9, 0),
            Version::V1_09_1 => (1, 9, 1),
            Version::V2_00_0 => (2, 0, 0),
        }
    }
}

fn get_version() -> Version {
    let file_path = {
        let mut buf = vec![0u16; MAX_PATH as usize];
        unsafe { GetModuleFileNameW(GetModuleHandleW(PCWSTR(null_mut())), &mut buf) };
        U16CString::from_vec_truncate(buf)
    };

    let mut version_info_size =
        unsafe { GetFileVersionInfoSizeW(PCWSTR(file_path.as_ptr()), null_mut()) };
    let mut version_info_buf = vec![0u8; version_info_size as usize];
    unsafe {
        GetFileVersionInfoW(
            PCWSTR(file_path.as_ptr()),
            0,
            version_info_size,
            version_info_buf.as_mut_ptr() as _,
        )
    };

    let mut version_info: *mut VS_FIXEDFILEINFO = null_mut();
    unsafe {
        VerQueryValueW(
            version_info_buf.as_ptr() as _,
            PCWSTR(widestring::U16CString::from_str("\\\\\0").unwrap().as_ptr()),
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
