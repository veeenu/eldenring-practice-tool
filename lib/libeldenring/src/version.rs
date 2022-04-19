use std::lazy::SyncLazy;
use std::ptr::null_mut;

use log::*;
use widestring::U16CString;
use windows::core::PCWSTR;
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};

pub enum Version {
    V1_02_0,
    V1_02_1,
    V1_02_2,
    V1_02_3,
    V1_03_0,
    V1_03_1,
    V1_03_2,
    V1_04_0,
}

pub static VERSION: SyncLazy<Version> = SyncLazy::new(get_version);

impl From<(u32, u32, u32)> for Version {
    fn from(v: (u32, u32, u32)) -> Self {
        match v {
            (1, 2, 0) => Version::V1_02_0,
            (1, 2, 1) => Version::V1_02_1,
            (1, 2, 2) => Version::V1_02_2,
            (1, 2, 3) => Version::V1_02_3,
            (1, 3, 0) => Version::V1_03_0,
            (1, 3, 1) => Version::V1_03_1,
            (1, 3, 2) => Version::V1_03_2,
            (1, 4, 0) => Version::V1_04_0,
            _ => {
                error!("Unrecognized version {}.{:02}.{}", v.0, v.1, v.2);
                panic!()
            }
        }
    }
}

fn get_version() -> Version {
    let file_path = {
        let mut buf = vec![0u16; MAX_PATH as usize];
        unsafe { GetModuleFileNameW(GetModuleHandleW(PCWSTR(null_mut())), &mut buf) };
        U16CString::from_vec_truncate(buf)
    };
    info!("{:?}", file_path);

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
    info!("VerQueryValueW");
    unsafe {
        VerQueryValueW(
            version_info_buf.as_ptr() as _,
            PCWSTR(widestring::U16CString::from_str("\\\\\0").unwrap().as_ptr()),
            &mut version_info as *mut *mut _ as _,
            &mut version_info_size,
        )
    };
    info!("Version info is {:p}", version_info);
    let version_info = unsafe { version_info.as_ref().unwrap() };
    let major = (version_info.dwFileVersionMS >> 16) & 0xffff;
    let minor = (version_info.dwFileVersionMS) & 0xffff;
    let patch = (version_info.dwFileVersionLS >> 16) & 0xffff;

    info!("Version {} {} {}", major, minor, patch);
    Version::from((major, minor, patch))
}
