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
mod widgets;

pub mod util;

use std::ffi::c_void;

use hudhook::hooks::dx12::ImguiDx12Hooks;
use hudhook::tracing::{error, trace};
use hudhook::{eject, Hudhook};
use libeldenring::codegen::base_addresses::BaseAddresses;
use libeldenring::version;
use once_cell::sync::Lazy;
use pkg_version::*;
use practice_tool::PracticeTool;
use practice_tool_core::update::{Update, Version};
use practice_tool_core_windows::dinput8::{system_direct_input8_create, FDirectInput8Create};
use practice_tool_core_windows::startup::{on_process_attach, Events};
use practice_tool_core_windows::xinput::hook_xinput;
use windows::core::{GUID, HRESULT};
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

/// Events coordinating the copies of the tool and its exe.
pub fn events() -> Events {
    Events::new("jdsd_er_practice_tool")
}

/// Checks GitHub for a newer release of the tool.
pub fn check_update() -> Update {
    Update::check(
        "veeenu/eldenring-practice-tool",
        Version::new(pkg_version_major!(), pkg_version_minor!(), pkg_version_patch!()),
    )
}

static DIRECTINPUT8CREATE: Lazy<FDirectInput8Create> = Lazy::new(|| {
    let directinput8create = system_direct_input8_create();

    unsafe { apply_no_logo() };

    directinput8create
});

#[no_mangle]
unsafe extern "system" fn DirectInput8Create(
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
    let offset = BaseAddresses::from(version::get_version()).func_remove_intro_screens;

    let ptr = (module_base.0 as usize + offset) as *mut [u8; 2];
    let mut old = PAGE_PROTECTION_FLAGS(0);
    if *ptr == [0x74, 0x53] && VirtualProtect(ptr as _, 2, PAGE_EXECUTE_READWRITE, &mut old).is_ok()
    {
        (*ptr) = [0x90, 0x90];
        VirtualProtect(ptr as _, 2, old, &mut old).ok();
    }
}

unsafe fn apply_event_patch() {
    let module_base = GetModuleHandleW(None).unwrap();

    let offset_1 = BaseAddresses::from(version::get_version()).event_patch1;
    let offset_2 = BaseAddresses::from(version::get_version()).event_patch2;

    let ptr_1 = (module_base.0 as usize + offset_1) as *mut [u8; 2];
    let mut old_1 = PAGE_PROTECTION_FLAGS(0);
    if *ptr_1 == [0x32, 0xC0]
        && VirtualProtect(ptr_1 as _, 2, PAGE_EXECUTE_READWRITE, &mut old_1).is_ok()
    {
        (*ptr_1) = [0xB0, 0x01];
        VirtualProtect(ptr_1 as _, 2, old_1, &mut old_1).ok();
    }

    let ptr_2 = (module_base.0 as usize + offset_2) as *mut [u8; 2];
    let mut old_2 = PAGE_PROTECTION_FLAGS(0);
    if *ptr_2 == [0x32, 0xC0]
        && VirtualProtect(ptr_2 as _, 2, PAGE_EXECUTE_READWRITE, &mut old_2).is_ok()
    {
        (*ptr_2) = [0xB0, 0x01];
        VirtualProtect(ptr_2 as _, 2, old_2, &mut old_2).ok();
    }
}

unsafe fn apply_font_patch() {
    let module_base = GetModuleHandleW(None).unwrap();
    let offset = BaseAddresses::from(version::get_version()).font_patch;

    let ptr = (module_base.0 as usize + offset) as *mut u8;
    let mut old = PAGE_PROTECTION_FLAGS(0);
    if *ptr == 0x48 && VirtualProtect(ptr as _, 1, PAGE_EXECUTE_READWRITE, &mut old).is_ok() {
        (*ptr) = 0xC3;
        VirtualProtect(ptr as _, 1, old, &mut old).ok();
    }
}

fn start_practice_tool(hmodule: HINSTANCE) {
    let practice_tool = PracticeTool::new();

    unsafe {
        apply_event_patch(); // Needed for event draw
        apply_font_patch(); // Needed for event draw & altimeter
    }

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

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn DllMain(
    hmodule: HINSTANCE,
    reason: u32,
    reserved: *mut c_void,
) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        trace!("DllMain()");
        if version::check_version().is_err() {
            return false;
        }

        Lazy::force(&DIRECTINPUT8CREATE);
        if let Err(e) = hook_xinput("xinput1_4.dll", None) {
            error!("{e}");
        }

        let hmodule_ptr = hmodule.0 as usize;
        on_process_attach(events(), !reserved.is_null(), move || {
            start_practice_tool(HINSTANCE(hmodule_ptr as *mut c_void))
        });
    }

    true
}
