#![feature(once_cell)]
use std::ffi::c_void;
use std::sync::LazyLock;
use std::ptr::null_mut;

use libeldenring::prelude::*;

use u16cstr::*;
use widestring::*;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::System::SystemInformation::GetSystemDirectoryW;

type FnDirectInput8Create = unsafe extern "stdcall" fn(
    _: HINSTANCE,
    _: u32,
    _: *const c_void,
    _: *const *const c_void,
    _: *const c_void,
) -> HRESULT;

type FnHResult = unsafe extern "stdcall" fn() -> HRESULT;
type FnGetClassObject =
    unsafe extern "stdcall" fn(*const c_void, *const c_void, *const c_void) -> HRESULT;

static SYMBOLS: LazyLock<(
    FnDirectInput8Create,
    FnHResult,
    FnGetClassObject,
    FnHResult,
    FnHResult,
)> = LazyLock::new(|| unsafe {
    apply_patch();
    let mut sys_path = vec![0u16; 320];
    let len = GetSystemDirectoryW(&mut sys_path) as u32;

    let sys_path = format!(
        "{}\\dinput8.dll",
        U16CString::from_ptr_truncate(sys_path.as_ptr(), len as usize).to_string_lossy()
    );
    let sys_path = U16CString::from_str(&sys_path).unwrap();

    let module = LoadLibraryW(PCWSTR(sys_path.as_ptr() as _));

    (
        std::mem::transmute(
            GetProcAddress(module, PCSTR("DirectInput8Create\0".as_ptr())).unwrap(),
        ),
        std::mem::transmute(GetProcAddress(module, PCSTR("DllCanUnloadNow\0".as_ptr())).unwrap()),
        std::mem::transmute(GetProcAddress(module, PCSTR("DllGetClassObject\0".as_ptr())).unwrap()),
        std::mem::transmute(GetProcAddress(module, PCSTR("DllRegisterServer\0".as_ptr())).unwrap()),
        std::mem::transmute(
            GetProcAddress(module, PCSTR("DllUnregisterServer\0".as_ptr())).unwrap(),
        ),
    )
});

unsafe fn apply_patch() {
    let module_base = GetModuleHandleW(PCWSTR(null_mut()));
    let offset = base_addresses::BaseAddresses::from(*VERSION).func_remove_intro_screens;

    let ptr = (module_base.0 as usize + offset) as *mut [u8; 2];
    let mut old = PAGE_PROTECTION_FLAGS(0);
    if *ptr == [0x74, 0x53] {
        VirtualProtect(ptr as _, 2, PAGE_EXECUTE_READWRITE, &mut old);
        (*ptr) = [0x90, 0x90];
        VirtualProtect(ptr as _, 2, old, &mut old);
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn DirectInput8Create(
    a: HINSTANCE,
    b: u32,
    c: *const c_void,
    d: *const *const c_void,
    e: *const c_void,
) -> HRESULT {
    (SYMBOLS.0)(a, b, c, d, e)
}

// #[no_mangle]
// pub unsafe extern "stdcall" fn DllCanUnloadNow() -> HRESULT {
//     (SYMBOLS.1)()
// }
//
// #[no_mangle]
// pub unsafe extern "stdcall" fn DllGetClassObject(
//     a: *const c_void,
//     b: *const c_void,
//     c: *const c_void,
// ) -> HRESULT {
//     (SYMBOLS.2)(a, b, c)
// }
//
// #[no_mangle]
// pub unsafe extern "stdcall" fn DllRegisterServer() -> HRESULT {
//     (SYMBOLS.3)()
// }
//
// #[no_mangle]
// pub unsafe extern "stdcall" fn DllUnregisterServer() -> HRESULT {
//     (SYMBOLS.4)()
// }

// #[no_mangle]
// unsafe extern "C" fn DllMain(
//     _hmodule: windows::Win32::Foundation::HINSTANCE,
//     reason: u32,
//     _: *mut std::ffi::c_void,
// ) -> BOOL {
//     if reason == DLL_PROCESS_ATTACH {
//         apply_patch();
//     }
//
//     BOOL(1)
// }
