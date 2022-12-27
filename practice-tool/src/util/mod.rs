mod vk;

use std::ffi::OsString;
use std::fmt::Display;
use std::os::windows::prelude::OsStringExt;
use std::path::PathBuf;

use log::*;
use serde::Deserialize;
pub(crate) use vk::*;
use windows::core::PCSTR;
use windows::Win32::Foundation::{GetLastError, HINSTANCE, MAX_PATH};
use windows::Win32::System::LibraryLoader::{
    GetModuleFileNameW, GetModuleHandleExA, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};

/// Returns the path of the implementor's DLL.
pub fn get_dll_path() -> Option<PathBuf> {
    let mut hmodule: HINSTANCE = Default::default();
    // SAFETY
    // This is reckless, but it should never fail, and if it does, it's ok to crash
    // and burn.
    let gmh_result = unsafe {
        GetModuleHandleExA(
            GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT | GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
            PCSTR("DllMain\0".as_ptr()),
            &mut hmodule,
        )
    };

    if gmh_result.0 == 0 {
        error!("get_dll_path: GetModuleHandleExA error: {:x}", unsafe { GetLastError().0 },);
        return None;
    }

    let mut sz_filename = [0u16; MAX_PATH as _];
    // SAFETY
    // pointer to sz_filename always defined and MAX_PATH bounds manually checked
    let len = unsafe { GetModuleFileNameW(hmodule, &mut sz_filename) } as usize;

    Some(OsString::from_wide(&sz_filename[..len]).into())
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(try_from = "String")]
pub(crate) struct KeyState(i32);

impl Display for KeyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_key_repr(self.0).unwrap_or("N/A"))
    }
}

impl KeyState {
    pub(crate) fn new(vkey: i32) -> Self {
        KeyState(vkey)
    }

    pub(crate) fn keyup(&self, ui: &imgui::Ui) -> bool {
        !ui.io().want_capture_keyboard && ui.is_key_index_released(self.0)
    }

    pub(crate) fn keydown(&self, ui: &imgui::Ui) -> bool {
        !ui.io().want_capture_keyboard && ui.is_key_index_pressed(self.0)
    }

    pub(crate) fn is_key_down(&self, ui: &imgui::Ui) -> bool {
        !ui.io().want_capture_keyboard && ui.is_key_index_down(self.0)
    }
}

impl TryFrom<String> for KeyState {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match get_key_code(&value) {
            Some(key_code) => Ok(KeyState::new(key_code)),
            None => Err(format!("\"{}\" is not a valid key code", value)),
        }
    }
}
