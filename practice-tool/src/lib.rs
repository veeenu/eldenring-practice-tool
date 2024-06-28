// johndisandonato's Elden Ring Practice Tool
// Copyright (C) 2022-2024  johndisandonato <https://github.com/veeenu>
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

mod config;
mod practice_tool;
mod update;
mod widgets;

pub mod util;

use std::ffi::c_void;
use std::time::{Duration, Instant};
use std::{mem, ptr, thread};

use hudhook::hooks::dx12::ImguiDx12Hooks;
use hudhook::tracing::error;
use hudhook::{eject, Hudhook};
use libeldenring::codegen::base_addresses::BaseAddresses;
use libeldenring::version::VERSION;
use once_cell::sync::Lazy;
use practice_tool::PracticeTool;
use windows::core::{s, w, GUID, HRESULT, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, MAX_PATH};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_RSHIFT};

type FDirectInput8Create = unsafe extern "stdcall" fn(
    hinst: HINSTANCE,
    dwversion: u32,
    riidltf: *const GUID,
    ppvout: *mut *mut c_void,
    punkouter: HINSTANCE,
) -> HRESULT;

static DIRECTINPUT8CREATE: Lazy<FDirectInput8Create> = Lazy::new(|| unsafe {
    let mut dinput8_path = [0u16; MAX_PATH as usize];
    let count = GetSystemDirectoryW(Some(&mut dinput8_path)) as usize;

    // If count == 0, this will be fun
    ptr::copy_nonoverlapping(w!("\\dinput8.dll").0, dinput8_path[count..].as_mut_ptr(), 12);

    let dinput8 = LoadLibraryW(PCWSTR(dinput8_path.as_ptr())).unwrap();
    let directinput8create = mem::transmute::<
        Option<unsafe extern "system" fn() -> isize>,
        unsafe extern "stdcall" fn(
            HINSTANCE,
            u32,
            *const GUID,
            *mut *mut c_void,
            HINSTANCE,
        ) -> HRESULT,
    >(GetProcAddress(dinput8, s!("DirectInput8Create")));

    apply_no_logo();

    directinput8create
});

#[no_mangle]
unsafe extern "stdcall" fn DirectInput8Create(
    hinst: HINSTANCE,
    dwversion: u32,
    riidltf: *const GUID,
    ppvout: *mut *mut c_void,
    punkouter: HINSTANCE,
) -> HRESULT {
    (DIRECTINPUT8CREATE)(hinst, dwversion, riidltf, ppvout, punkouter)
}

unsafe fn apply_no_logo() {
    let module_base = GetModuleHandleW(None).unwrap();
    let offset = BaseAddresses::from(*VERSION).func_remove_intro_screens;

    let ptr = (module_base.0 as usize + offset) as *mut [u8; 2];
    let mut old = PAGE_PROTECTION_FLAGS(0);
    if *ptr == [0x74, 0x53] && VirtualProtect(ptr as _, 2, PAGE_EXECUTE_READWRITE, &mut old).is_ok()
    {
        (*ptr) = [0x90, 0x90];
        VirtualProtect(ptr as _, 2, old, &mut old).ok();
    }
}

fn start_practice_tool(hmodule: HINSTANCE) {
    let practice_tool = PracticeTool::new();

    if let Err(e) = Hudhook::builder()
        .with::<ImguiDx12Hooks>(practice_tool)
        .with_hmodule(hmodule)
        .build()
        .apply()
    {
        error!("Couldn't apply hooks: {e:?}");
        eject();
    }
}

fn await_rshift() -> bool {
    let duration_threshold = Duration::from_secs(2);
    let check_window = Duration::from_secs(10);
    let poll_interval = Duration::from_millis(100);

    let start_time = Instant::now();
    let mut key_down_start: Option<Instant> = None;

    while start_time.elapsed() < check_window {
        let state = unsafe { GetAsyncKeyState(VK_RSHIFT.0 as i32) };
        let key_down = state < 0;

        match (key_down, key_down_start) {
            (true, None) => {
                key_down_start = Some(Instant::now());
            },
            (true, Some(start)) => {
                if start.elapsed() >= duration_threshold {
                    return true;
                }
            },
            (false, _) => {
                key_down_start = None;
            },
        }

        thread::sleep(poll_interval);
    }

    false
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn DllMain(hmodule: HINSTANCE, reason: u32, _: *mut c_void) {
    if reason == DLL_PROCESS_ATTACH {
        thread::spawn(move || {
            if util::get_dll_path()
                .and_then(|path| {
                    path.file_name().map(|s| s.to_string_lossy().to_lowercase() == "dinput8.dll")
                })
                .unwrap_or(false)
            {
                if await_rshift() {
                    start_practice_tool(hmodule)
                }
            } else {
                start_practice_tool(hmodule)
            }
        });
    }
}
