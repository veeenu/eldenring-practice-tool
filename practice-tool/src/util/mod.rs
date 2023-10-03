mod vk;

use std::ffi::OsString;
use std::fmt::Display;
use std::os::windows::prelude::OsStringExt;
use std::path::PathBuf;

use hudhook::tracing::*;
use serde::Deserialize;
pub(crate) use vk::*;
use windows::core::PCSTR;
use windows::Win32::Foundation::{HMODULE, MAX_PATH};
use windows::Win32::System::LibraryLoader::{
    GetModuleFileNameW, GetModuleHandleExA, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};

/// Returns the path of the implementor's DLL.
pub fn get_dll_path() -> Option<PathBuf> {
    let mut hmodule: HMODULE = Default::default();
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

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(try_from = "String")]
pub(crate) struct KeyState(i32, Option<i32>);

impl Display for KeyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_key_repr(self.0).unwrap_or("N/A"))
    }
}

impl KeyState {
    pub(crate) fn new(vkey: i32, modifier: Option<i32>) -> Self {
        KeyState(vkey, modifier)
    }

    pub(crate) fn is_modifier_down(&self, ui: &imgui::Ui) -> bool {
        self.1.map(|m| ui.is_key_index_down(m as u32)).unwrap_or(true)
    }

    pub(crate) fn keyup(&self, ui: &imgui::Ui) -> bool {
        ui.is_key_index_released(self.0 as u32) && self.is_modifier_down(ui)
    }

    pub(crate) fn keydown(&self, ui: &imgui::Ui) -> bool {
        ui.is_key_index_pressed(self.0 as u32) && self.is_modifier_down(ui)
    }

    pub(crate) fn is_key_down(&self, ui: &imgui::Ui) -> bool {
        ui.is_key_index_down(self.0 as u32) && self.is_modifier_down(ui)
    }
}

impl TryFrom<String> for KeyState {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut pieces = value.rsplit('+');
        let key_value = pieces.next().ok_or_else(|| "Empty string for key code".to_string())?;
        let modifier = pieces
            .next()
            .map(|modifier| {
                get_key_code(modifier)
                    .ok_or_else(|| format!("\"{modifier}\" is not a valid key code"))
            })
            .transpose()?;

        match get_key_code(key_value) {
            Some(key_code) => Ok(KeyState::new(key_code, modifier)),
            None => Err(format!("\"{}\" is not a valid key code", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_state() {
        println!("{:?}", KeyState::try_from("rshift+o".to_string()));
        println!("{:?}", KeyState::try_from("rshift".to_string()))
    }
}
