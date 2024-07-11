use std::env;
use std::path::{Path, PathBuf};

use practice_tool_tasks::codegen::{self, aob_direct, aob_indirect_twice};
use textwrap::dedent;

fn patches_paths() -> impl Iterator<Item = PathBuf> {
    let base_path = PathBuf::from(
        env::var("ER_PATCHES_PATH").unwrap_or_else(|_| panic!("{}", dedent(r"
            ER_PATCHES_PATH environment variable undefined.
            Check the documentation: https://github.com/veeenu/eldenring-practice-tool/README.md#building
        "))),
    );
    base_path
        .read_dir()
        .expect("Couldn't scan patches directory")
        .map(Result::unwrap)
        .map(|dir| dir.path().join("Game").join("eldenring.exe"))
}

fn base_addresses_rs_path() -> PathBuf {
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
    let aobs = &[
        aob_indirect_twice(
            "BulletMan",
            &["48 8B 0D ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8D 44 24 ?? 48 89 44 24 ?? 48 89 7C 24 ?? \
               C7 44 24 ?? ?? ?? ?? ?? 48"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "ChrDbgFlags",
            &["?? 80 3D ?? ?? ?? ?? 00 0F 85 ?? ?? ?? ?? 32 C0 48"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "CSFD4VirtualMemoryFlag",
            &["48 8B 3D ?? ?? ?? ?? 48 85 FF 74 ?? 48 8B 49"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "CSFlipper",
            &["48 8B 0D ?? ?? ?? ?? 80 BB D7 00 00 00 00 0F 84 CE 00 00 00 48 85 C9 75 2E"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "CSLuaEventManager",
            &[
                "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 ?? 41 BE 01 00 00 00 44 89 74 24",
                "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 ?? 41 BE 01 00 00 00 44 89 75 83",
            ],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "CSMenuMan",
            &["E8 ?? ?? ?? ?? 4C 8B F8 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 0D"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "CSMenuManImp",
            &["48 8B 0D ?? ?? ?? ?? 48 8B 49 08 E8 ?? ?? ?? ?? 48 8B D0 48 8B CE E8 ?? ?? ?? ??"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "CSNetMan",
            &["48 8B 0D ?? ?? ?? ?? 48 85 C9 74 5E 48 8B 89 ?? ?? ?? ?? B2 01"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "CSRegulationManager",
            &["48 8B 0D ?? ?? ?? ?? 48 85 C9 74 0B 4C 8B C0 48 8B D7"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "CSSessionManager",
            &["48 8B 05 ?? ?? ?? ?? 48 89 9C 24 E8 00 00 00 48 89 ?? 24 B0 00 00 00 ?? 89 ?? 24 \
               A8 00 00 00 ?? 89 ?? 24 A0 00 00 00 48 85 C0"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "DamageCtrl",
            &["48 8B 05 ?? ?? ?? ?? 49 8B D9 49 8B F8 48 8B F2 48 85 C0 75 2E"],
            3,
            7,
            true,
        ),
        // aob_indirect_twice("FieldArea", "48 8B 3D ?? ?? ?? ?? 48 85 FF 0F 84 ?? ?? ?? ?? 45 38
        // 66 34",3,7),
        aob_indirect_twice(
            "FieldArea",
            &["48 8B 0D ?? ?? ?? ?? 48 ?? ?? ?? 44 0F B6 61 ?? E8 ?? ?? ?? ?? 48 63 87 ?? ?? ?? \
               ?? 48 ?? ?? ?? 48 85 C0"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "GameDataMan",
            &["48 8B 05 ?? ?? ?? ?? 48 85 C0 74 05 48 8B 40 58 C3 C3"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "GameMan",
            &["48 8B 1D ?? ?? ?? ?? 48 8B F8 48 85 DB 74 18 4C 8B 03"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "GlobalPos",
            &["48 8B 3D ?? ?? ?? ?? 33 DB 49 8B F0 4C 8B F1 48 85 FF"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "GroupMask",
            &["?? 80 3D ?? ?? ?? ?? 00 0F 10 00 0F 11 45 D0 0F 84 ?? ?? ?? ?? 80 3D"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "HitIns",
            &["48 8B 05 ?? ?? ?? ?? 48 8D 4C 24 ?? 48 89 4c 24 ?? 0F 10 44 24 70"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "HitInsHitboxOffset",
            &["0F B6 25 ?? ?? ?? ?? 44 0F B6 3D ?? ?? ?? ?? E8 ?? ?? ?? ?? 0F B6 F8"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "MapItemMan",
            &["48 8B 0D ?? ?? ?? ?? C7 44 24 50 FF FF FF FF C7 45 A0 FF FF FF FF 48 85 C9 75 2E"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "MenuManIns",
            &["48 8b 0d ?? ?? ?? ?? 48 8b 53 08 48 8b 92 d8 00 00 00 48 83 c4 20 5b"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "MsgRepository",
            &["48 8B 3D ?? ?? ?? ?? 44 0F B6 30 48 85 FF 75 26"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "SoloParamRepository",
            &["48 8B 0D ?? ?? ?? ?? 48 85 C9 0F 84 ?? ?? ?? ?? 45 33 C0 BA 8D 00 00 00 E8"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "WorldChrMan",
            &[
                "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 0F 48 39 88 ?? ?? ?? ?? 75 06 89 B1 5C 03 00 00 \
                 0F 28 05 ?? ?? ?? ?? 4C 8D 45 E7",
                "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 0F 48 39 88",
            ],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "WorldChrManDbg",
            &["48 8B 0D ?? ?? ?? ?? 89 5C 24 20 48 85 C9 74 12 B8 ?? ?? ?? ?? 8B D8"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "WorldChrManImp",
            &[
                "48 8B 05 ?? ?? ?? ?? 48 85 C0 74 0F 48 39 88 ?? ?? ?? ?? 75 06 89 B1 5C 03 00 00 \
                 0F 28 05 ?? ?? ?? ?? 4C 8D 45 E7",
                "48 8B 35 ?? ?? ?? ?? 48 85 F6 ?? ?? BB 01 00 00 00 89 5C 24 20 48 8B B6",
            ],
            3,
            7,
            true,
        ),
        aob_direct(
            "FuncItemSpawn",
            &["48 8B C4 56 57 41 56 48 81 EC ?? ?? ?? ?? 48 C7 44 24 ?? ?? ?? ?? ?? 48 89 58 ?? \
               48 89 68 ?? 48 8B 05 ?? ?? ?? ?? 48 33 C4 48 89 84 24 ?? ?? ?? ?? 41 0F B6 F9"],
            true,
        ),
        aob_direct(
            "FuncItemInject",
            &[
                "40 55 56 57 41 54 41 55 41 56 41 57 48 8D 6C 24 B0 48 81 EC 50 01 00 00 48 C7 45 \
                 C0 FE FF FF FF", // 1.02
                "40 55 56 57 41 54 41 55 41 56 41 57 48 8d ac 24 ?? ?? ?? ?? 48 81 ec ?? ?? ?? ?? \
                 48 c7 45 ?? ?? ?? ?? ?? 48 89 9c 24 ?? ?? ?? ?? 48 8b 05 ?? ?? ?? ?? 48 33 c4 48 \
                 89 85 ?? ?? ?? ?? 44 89 4c 24", // 1.03
                "40 55 56 57 41 54 41 55 41 56 41 57 48 8D AC 24 70 FF FF FF 48 81 EC 90 01 00 00 \
                 48 C7 45 C8 FE FF FF FF 48 89 9C 24 D8",
            ],
            true,
        ), // 1.04
        aob_direct(
            "FuncRemoveIntroScreens",
            &["74 53 48 8B 05 ?? ?? ?? ?? 48 85 C0 75 2E 48 8D 0D ?? ?? ?? ?? E8 ?? ?? ?? ?? 4C \
               8B C8"],
            true,
        ),
        aob_direct("FuncDbgActionForce", &["48 8B 41 08 0F BE 80 ?? E9 00 00 48 8D 64"], true),
        aob_direct("LuaWarp", &["C3 ?? ?? ?? ?? ?? ?? 57 48 83 EC ?? 48 8B FA 44"], true),
        aob_direct("CurrentTarget", &["48 8B 48 08 49 89 8D ?? ?? ?? ?? 49 8B CE E8"], true),
        aob_indirect_twice(
            "BaseFPS",
            &["48 8B 0D ?? ?? ?? ?? 48 85 C9 75 2E 48 8D 0D ?? ?? ?? ?? E8 ?? ?? ?? ?? 4C 8B C8 \
               4C 8D 05 ?? ?? ?? ?? BA ?? ?? ?? ?? 48 8D 0D ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8B 0D \
               ?? ?? ?? ?? 48 83 C1 20 E8 ?? ?? ?? ??"],
            3,
            7,
            true,
        ),
        aob_indirect_twice(
            "BaseAnim",
            &["48 89 0D ?? ?? ?? ?? 8D 46 9C 83 F8 21 77 37 83 FE 66 74 26 83 FE 70 74 15 83 FE \
               7C 0F 85 ?? ?? ?? ?? 48 8D BB ?? ?? ?? ?? E9 ?? ?? ?? ?? 48 8D BB ?? ?? ?? ??"],
            3,
            7,
            true,
        ),
    ];

    codegen::codegen_base_addresses(base_addresses_rs_path(), patches_paths(), aobs)
}
