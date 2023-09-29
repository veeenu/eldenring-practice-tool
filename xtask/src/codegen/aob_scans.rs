use std::collections::HashSet;
use std::env;
use std::ffi::c_void;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::LazyLock;

use heck::AsSnakeCase;
use rayon::prelude::*;
use textwrap::dedent;
use widestring::U16CString;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, BOOL, CHAR, DBG_CONTINUE};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
};
use windows::Win32::System::Diagnostics::Debug::{
    ContinueDebugEvent, ReadProcessMemory, WaitForDebugEventEx, DEBUG_EVENT,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE,
};
use windows::Win32::System::Threading::{CreateProcessW, OpenProcess, *};

// Indirect AoB patterns -- grab the bytes 3-7 as a u32 offset.
static AOBS: LazyLock<Vec<(&str, Vec<&str>)>> = LazyLock::new(|| {
    vec![
        ("BulletMan", vec![
            "48 8B 0D ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8D 44 24 ?? 48 89 44 24 ?? 48 89 7C 24 ?? C7 \
             44 24 ?? ?? ?? ?? ?? 48",
        ]),
        ("ChrDbgFlags", vec!["?? 80 3D ?? ?? ?? ?? 00 0F 85 ?? ?? ?? ?? 32 C0 48"]),
        ("CSFD4VirtualMemoryFlag", vec!["48 8B 3D ?? ?? ?? ?? 48 85 FF 74 ?? 48 8B 49"]),
        ("CSFlipper", vec![
            "48 8B 0D ?? ?? ?? ?? 80 BB D7 00 00 00 00 0F 84 CE 00 00 00 48 85 C9 75 2E",
        ]),
        ("CSLuaEventManager", vec![
            "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 ?? 41 BE 01 00 00 00 44 89 74 24",
            "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 ?? 41 BE 01 00 00 00 44 89 75 83",
        ]),
        ("CSMenuMan", vec!["E8 ?? ?? ?? ?? 4C 8B F8 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 0D"]),
        ("CSMenuManImp", vec![
            "48 8B 0D ?? ?? ?? ?? 48 8B 49 08 E8 ?? ?? ?? ?? 48 8B D0 48 8B CE E8 ?? ?? ?? ??",
        ]),
        ("CSNetMan", vec!["48 8B 0D ?? ?? ?? ?? 48 85 C9 74 5E 48 8B 89 ?? ?? ?? ?? B2 01"]),
        ("CSRegulationManager", vec!["48 8B 0D ?? ?? ?? ?? 48 85 C9 74 0B 4C 8B C0 48 8B D7"]),
        ("CSSessionManager", vec![
            "48 8B 05 ?? ?? ?? ?? 48 89 9C 24 E8 00 00 00 48 89 ?? 24 B0 00 00 00 ?? 89 ?? 24 A8 \
             00 00 00 ?? 89 ?? 24 A0 00 00 00 48 85 C0",
        ]),
        ("DamageCtrl", vec!["48 8B 05 ?? ?? ?? ?? 49 8B D9 49 8B F8 48 8B F2 48 85 C0 75 2E"]),
        // ("FieldArea", "48 8B 3D ?? ?? ?? ?? 48 85 FF 0F 84 ?? ?? ?? ?? 45 38 66 34"),
        ("FieldArea", vec![
            "48 8B 0D ?? ?? ?? ?? 48 ?? ?? ?? 44 0F B6 61 ?? E8 ?? ?? ?? ?? 48 63 87 ?? ?? ?? ?? \
             48 ?? ?? ?? 48 85 C0",
        ]),
        ("GameDataMan", vec!["48 8B 05 ?? ?? ?? ?? 48 85 C0 74 05 48 8B 40 58 C3 C3"]),
        ("GameMan", vec!["48 8B 1D ?? ?? ?? ?? 48 8B F8 48 85 DB 74 18 4C 8B 03"]),
        ("GroupMask", vec!["?? 80 3D ?? ?? ?? ?? 00 0F 10 00 0F 11 45 D0 0F 84 ?? ?? ?? ?? 80 3D"]),
        ("HitIns", vec!["48 8B 05 ?? ?? ?? ?? 48 8D 4C 24 ?? 48 89 4c 24 ?? 0F 10 44 24 70"]),
        ("HitInsHitboxOffset", vec![
            "0F B6 25 ?? ?? ?? ?? 44 0F B6 3D ?? ?? ?? ?? E8 ?? ?? ?? ?? 0F B6 F8",
        ]),
        ("GlobalPos", vec!["48 8B 3D ?? ?? ?? ?? 33 DB 49 8B F0 4C 8B F1 48 85 FF"]),
        ("MapItemMan", vec![
            "48 8B 0D ?? ?? ?? ?? C7 44 24 50 FF FF FF FF C7 45 A0 FF FF FF FF 48 85 C9 75 2E",
        ]),
        ("MenuManIns", vec![
            "48 8b 0d ?? ?? ?? ?? 48 8b 53 08 48 8b 92 d8 00 00 00 48 83 c4 20 5b",
        ]),
        ("MsgRepository", vec!["48 8B 3D ?? ?? ?? ?? 44 0F B6 30 48 85 FF 75 26"]),
        ("SoloParamRepository", vec![
            "48 8B 0D ?? ?? ?? ?? 48 85 C9 0F 84 ?? ?? ?? ?? 45 33 C0 BA 8D 00 00 00 E8",
        ]),
        ("WorldChrMan", vec![
            "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 0F 48 39 88 ?? ?? ?? ?? 75 06 89 B1 5C 03 00 00 0F \
             28 05 ?? ?? ?? ?? 4C 8D 45 E7",
            "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 0F 48 39 88",
        ]),
        ("WorldChrManDbg", vec![
            "48 8B 0D ?? ?? ?? ?? 89 5C 24 20 48 85 C9 74 12 B8 ?? ?? ?? ?? 8B D8",
        ]),
        ("WorldChrManImp", vec![
            "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 0F 48 39 88 ?? ?? ?? ?? 75 06 89 B1 5C 03 00 00 0F \
             28 05 ?? ?? ?? ?? 4C 8D 45 E7",
            "48 8B 35 ?? ?? ?? ?? 48 85 F6 ?? ?? BB 01 00 00 00 89 5C 24 20 48 8B B6",
        ]),
    ]
});

// Direct AoB patterns -- grab the position of the match. For static functions
static AOBS_DIRECT: LazyLock<Vec<(&str, Vec<&str>)>> = LazyLock::new(|| {
    vec![
        ("FuncItemSpawn", vec![
            "48 8B C4 56 57 41 56 48 81 EC ?? ?? ?? ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 89 58 ?? 48 \
             89 68 ?? 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 84 24 ?? ?? ?? ?? 41 0F B6 F9",
        ]),
        ("FuncItemInject", vec![
            "40 55 56 57 41 54 41 55 41 56 41 57 48 8D 6C 24 B0 48 81 EC 50 01 00 00 48 C7 45 C0 \
             FE FF FF FF", // 1.02
            "40 55 56 57 41 54 41 55 41 56 41 57 48 8d ac 24 ?? ?? ?? ?? 48 81 ec ?? ?? ?? ?? 48 \
             c7 45 ?? ?? ?? ?? ?? 48 89 9c 24 ?? ?? ?? ?? 48 8b 05 ?? ?? ?? ?? 48 33 c4 48 89 85 \
             ?? ?? ?? ?? 44 89 4c 24", // 1.03
            "40 55 56 57 41 54 41 55 41 56 41 57 48 8D AC 24 70 FF FF FF 48 81 EC 90 01 00 00 48 \
             C7 45 C8 FE FF FF FF 48 89 9C 24 D8",
        ]), // 1.04
        ("FuncRemoveIntroScreens", vec![
            "74 53 48 8B 05 ?? ?? ?? ?? 48 85 C0 75 2E 48 8D 0D ?? ?? ?? ?? E8 ?? ?? ?? ?? 4C 8B \
             C8",
        ]),
        ("FuncDbgActionForce", vec!["48 8B 41 08 0F BE 80 ?? E9 00 00 48 8D 64"]),
        ("LuaWarp", vec!["C3 ?? ?? ?? ?? ?? ?? 57 48 83 EC ?? 48 8B FA 44"]),
        ("CurrentTarget", vec!["48 8B 48 08 49 89 8D ?? ?? ?? ?? 49 8B CE E8"]),
    ]
});

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Version(u32, u32, u32);

impl Version {
    fn to_fromsoft_string(self) -> String {
        format!("{}.{:02}.{}", self.0, self.1, self.2)
    }
}

struct VersionData {
    version: Version,
    aobs: Vec<(&'static str, usize)>,
}

fn szcmp(source: &[CHAR], s: &str) -> bool {
    source.iter().zip(s.chars()).all(|(a, b)| a.0 == b as u8)
}

fn into_needle(pattern: &str) -> Vec<Option<u8>> {
    pattern
        .split(' ')
        .map(|byte| match byte {
            "?" | "??" => None,
            x => u8::from_str_radix(x, 16).ok(),
        })
        .collect::<Vec<_>>()
}

fn naive_search(bytes: &[u8], pattern: &[Option<u8>]) -> Option<usize> {
    bytes.windows(pattern.len()).position(|wnd| {
        wnd.iter().zip(pattern.iter()).all(|(byte, pattern)| match pattern {
            Some(x) => byte == x,
            None => true,
        })
    })
}

fn read_base_module_data(proc_name: &str, pid: u32) -> Option<(usize, Vec<u8>)> {
    let module_snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid) };
    let mut module_entry =
        MODULEENTRY32 { dwSize: std::mem::size_of::<MODULEENTRY32>() as _, ..Default::default() };

    unsafe { Module32First(module_snapshot, &mut module_entry) };

    loop {
        if szcmp(&module_entry.szModule, proc_name) {
            let process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, true, pid) };
            let mut buf = vec![0u8; module_entry.modBaseSize as usize];
            let mut bytes_read = 0usize;
            unsafe {
                ReadProcessMemory(
                    process,
                    module_entry.modBaseAddr as *mut c_void,
                    buf.as_mut_ptr() as *mut c_void,
                    module_entry.modBaseSize as usize,
                    &mut bytes_read,
                )
            };
            println!("Read {:x} out of {:x} bytes", bytes_read, module_entry.modBaseSize);
            unsafe { CloseHandle(process) };
            return Some((module_entry.modBaseAddr as usize, buf));
        }
        if !unsafe { Module32Next(module_snapshot, &mut module_entry).as_bool() } {
            break;
        }
    }
    None
}

fn get_base_module_bytes(exe_path: &Path) -> Option<(usize, Vec<u8>)> {
    let mut process_info = PROCESS_INFORMATION::default();
    let startup_info =
        STARTUPINFOW { cb: std::mem::size_of::<STARTUPINFOW>() as _, ..Default::default() };

    let mut exe = U16CString::from_str(exe_path.to_str().unwrap()).unwrap().into_vec();
    exe.push(0);

    let process = unsafe {
        CreateProcessW(
            PCWSTR(exe.as_ptr()),
            PWSTR(null_mut()),
            null(),
            null(),
            BOOL::from(false),
            DEBUG_PROCESS | DETACHED_PROCESS,
            null(),
            PCWSTR(null()),
            &startup_info,
            &mut process_info,
        )
    };

    if !process.as_bool() {
        eprintln!("Could not create process: {:x}", unsafe { GetLastError() }.0);
        return None;
    }

    println!("Process handle={:x} pid={}", process_info.hProcess.0, process_info.dwProcessId);

    let mut debug_event = DEBUG_EVENT::default();

    loop {
        unsafe { WaitForDebugEventEx(&mut debug_event, 1000) };
        unsafe {
            ContinueDebugEvent(
                process_info.dwProcessId,
                process_info.dwThreadId,
                DBG_CONTINUE.0 as _,
            )
        };
        if debug_event.dwDebugEventCode.0 == 2 {
            break;
        }
    }

    let ret = read_base_module_data(
        exe_path.file_name().unwrap().to_str().unwrap(),
        process_info.dwProcessId,
    );

    unsafe { TerminateProcess(process_info.hProcess, 0) };

    ret
}

fn find_aobs(bytes: Vec<u8>) -> Vec<(&'static str, usize)> {
    let mut aob_offsets = AOBS
        .par_iter()
        .filter_map(|(name, aob)| {
            if let Some(r) = aob.iter().find_map(|aob| naive_search(&bytes, &into_needle(aob))) {
                Some((*name, r))
            } else {
                eprintln!("{name:24} not found");
                None
            }
        })
        .map(|offset| {
            (
                offset.0,
                offset.1,
                u32::from_le_bytes(bytes[offset.1 + 3..offset.1 + 7].try_into().unwrap()),
            )
        })
        .map(|offset| (offset.0, (offset.2 + 7) as usize + offset.1))
        .collect::<Vec<_>>();

    aob_offsets.sort_by(|a, b| a.0.cmp(b.0));

    let mut aob_offsets_direct = AOBS_DIRECT
        .par_iter()
        .filter_map(|(name, aob)| {
            if let Some(r) = aob.iter().find_map(|aob| naive_search(&bytes, &into_needle(aob))) {
                Some((*name, r))
            } else {
                eprintln!("{name:24} not found");
                None
            }
        })
        .collect::<Vec<_>>();

    aob_offsets_direct.sort_by(|a, b| a.0.cmp(b.0));

    aob_offsets.extend(aob_offsets_direct);

    aob_offsets
}

fn get_file_version(file: &Path) -> Version {
    let mut file_path = file.to_string_lossy().to_string();
    file_path.push(0 as char);
    let file_path = widestring::U16CString::from_str(file_path).unwrap();
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

    Version(major, minor, patch)
}

// Codegen routine
//

/// Generate the `BaseAddresses` struct.
fn codegen_base_addresses_struct() -> String {
    let mut generated = String::new();

    generated.push_str("// **********************************\n");
    generated.push_str("// *** AUTOGENERATED, DO NOT EDIT ***\n");
    generated.push_str("// **********************************\n");

    generated.push_str("#[derive(Debug)]\n");
    generated.push_str("pub struct BaseAddresses {\n");
    generated.push_str(
        &AOBS
            .iter()
            .map(|(name, _)| format!("    pub {}: usize,\n", AsSnakeCase(name)))
            .collect::<Vec<_>>()
            .join(""),
    );
    generated.push_str({
        &AOBS_DIRECT
            .iter()
            .map(|(name, _)| format!("    pub {}: usize,\n", AsSnakeCase(name)))
            .collect::<Vec<_>>()
            .join("")
    });
    generated.push_str("}\n\n");
    generated.push_str("impl BaseAddresses {\n");
    generated.push_str("    pub fn with_module_base_addr(self, base: usize) -> BaseAddresses {\n");
    generated.push_str("        BaseAddresses {\n");
    generated.push_str(
        &AOBS
            .iter()
            .map(|(name, _)| {
                format!("            {}: self.{} + base,\n", AsSnakeCase(name), AsSnakeCase(name))
            })
            .collect::<Vec<_>>()
            .join(""),
    );
    generated.push_str(
        &AOBS_DIRECT
            .iter()
            .map(|(name, _)| {
                format!("            {}: self.{} + base,\n", AsSnakeCase(name), AsSnakeCase(name))
            })
            .collect::<Vec<_>>()
            .join(""),
    );
    generated.push_str("        }\n    }\n}\n\n");
    generated
}

/// Generate `BaseAddresses` instances.
fn codegen_base_addresses_instances(ver: &Version, aobs: &[(&str, usize)]) -> String {
    use std::fmt::Write;
    let mut string = aobs.iter().fold(
        format!(
            "pub const BASE_ADDRESSES_{}_{:02}_{}: BaseAddresses = BaseAddresses {{\n",
            ver.0, ver.1, ver.2
        ),
        |mut o, (name, offset)| {
            writeln!(o, "    {}: 0x{:x},", AsSnakeCase(name), offset).unwrap();
            o
        },
    );
    string.push_str("};\n\n");
    string
}

/// Generate the `Version` enum and `From<Version> for BaseAddresses`.
fn codegen_version_enum(ver: &[VersionData]) -> String {
    use std::fmt::Write;
    let mut string = String::new();

    // pub enum Version

    string.push_str("#[derive(Clone, Copy)]\n");
    string.push_str("pub enum Version {\n");

    for v in ver {
        writeln!(string, "    V{}_{:02}_{},", v.version.0, v.version.1, v.version.2).unwrap();
    }

    string.push_str("}\n\n");

    // impl From<(u32, u32, u32)> for Version

    string.push_str("impl From<(u32, u32, u32)> for Version {\n");
    string.push_str("    fn from(v: (u32, u32, u32)) -> Self {\n");
    string.push_str("        match v {\n");

    for v in ver {
        let Version(maj, min, patch) = v.version;
        writeln!(
            string,
            "            ({maj}, {min}, {patch}) => Version::V{maj}_{min:02}_{patch},"
        )
        .unwrap();
    }

    string.push_str("            (maj, min, patch) => {\n");
    string.push_str(
        "                log::error!(\"Unrecognized version {maj}.{min:02}.{patch}\");\n",
    );
    string.push_str("                panic!()\n");
    string.push_str("            }\n");
    string.push_str("        }\n");
    string.push_str("    }\n");
    string.push_str("}\n\n");

    // impl From<Version> for BaseAddresses

    string.push_str("impl From<Version> for BaseAddresses {\n");
    string.push_str("    fn from(v: Version) -> Self {\n");
    string.push_str("        match v {\n");

    for v in ver {
        let Version(maj, min, patch) = v.version;
        let stem = format!("{maj}_{min:02}_{patch}");
        writeln!(string, "            Version::V{stem} => BASE_ADDRESSES_{stem},").unwrap();
    }

    string.push_str("        }\n");
    string.push_str("    }\n");
    string.push_str("}\n\n");

    string
}

fn patches_paths() -> impl Iterator<Item = PathBuf> {
    let base_path = PathBuf::from(
        env::var("ERPT_PATCHES_PATH").unwrap_or_else(|_| panic!("{}", dedent(r"
            ERPT_PATCHES_PATH environment variable undefined.
            Check the documentation: https://github.com/veeenu/eldenring-practice-tool/README.md#building
        "))),
    );
    base_path
        .read_dir()
        .expect("Couldn't scan patches directory")
        .map(Result::unwrap)
        .map(|dir| dir.path().join("Game").join("eldenring.exe"))
}

fn codegen_base_addresses_path() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
        .join("lib")
        .join("libeldenring")
        .join("src")
        .join("codegen")
        .join("base_addresses.rs")
}

pub(crate) fn get_base_addresses() {
    let mut processed_versions: HashSet<Version> = HashSet::new();

    let version_data = patches_paths()
        .filter(|p| p.exists())
        .filter_map(|exe| {
            let version = get_file_version(&exe);
            if processed_versions.contains(&version) {
                None
            } else {
                let exe = exe.canonicalize().unwrap();
                println!("\nVERSION {}: {:?}", version.to_fromsoft_string(), exe);

                let (_base_addr, bytes) = get_base_module_bytes(&exe).unwrap();
                let aobs = find_aobs(bytes);
                processed_versions.insert(version);
                Some(VersionData { version, aobs })
            }
        })
        .collect::<Vec<_>>();

    let mut codegen = codegen_base_addresses_struct();
    codegen.push_str(&codegen_version_enum(&version_data));

    let codegen = version_data.iter().fold(codegen, |mut o, i| {
        o.push_str(&codegen_base_addresses_instances(&i.version, &i.aobs));
        o
    });

    File::create(codegen_base_addresses_path()).unwrap().write_all(codegen.as_bytes()).unwrap();
}
