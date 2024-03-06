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

#![feature(lazy_cell)]
#![feature(const_fn_floating_point_arithmetic)]

mod config;
mod practice_tool;
mod util;
mod widgets;

use std::ffi::c_void;
use std::thread;

use hudhook::hooks::dx12::ImguiDx12Hooks;
use hudhook::tracing::{error, trace};
use hudhook::windows::Win32::Foundation::HINSTANCE;
use hudhook::windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use hudhook::{eject, Hudhook, ImguiRenderLoop};
use once_cell::sync::Lazy;
use practice_tool::PracticeTool;

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn DllMain(hmodule: HINSTANCE, reason: u32, _: *mut c_void) {
    if reason == DLL_PROCESS_ATTACH {
        thread::spawn(move || {
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
        });
    }
}
