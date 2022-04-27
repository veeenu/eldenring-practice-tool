#![allow(clippy::new_without_default)]

use std::fmt::Display;
use std::ptr::null_mut;

use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;

use crate::memedit::*;
use crate::prelude::base_addresses::BaseAddresses;
use crate::prelude::Version;

#[derive(Debug)]
pub struct Pointers {
    pub one_shot: Bitflag<u8>,
    pub no_damage: Bitflag<u8>,
    pub no_dead: Bitflag<u8>,
    pub no_hit: Bitflag<u8>,
    pub no_goods_consume: Bitflag<u8>,
    pub no_stamina_consume: Bitflag<u8>,
    pub no_fp_consume: Bitflag<u8>,
    pub no_arrows_consume: Bitflag<u8>,
    pub no_attack: Bitflag<u8>,
    pub no_move: Bitflag<u8>,
    pub no_update_ai: Bitflag<u8>,
    pub no_ashes_of_war_fp_consume: Bitflag<u8>,

    pub collision: Bitflag<u8>,

    pub torrent_no_dead: Bitflag<u8>,
    pub torrent_gravity: Bitflag<u8>,
    pub torrent_collision: Bitflag<u8>,

    pub all_no_dead: Bitflag<u8>,

    pub character_stats: PointerChain<CharacterStats>,
    pub runes: PointerChain<u32>,
    pub igt: PointerChain<usize>,

    pub weapon_hitbox1: Bitflag<u8>, // debug sphere 1
    pub weapon_hitbox2: Bitflag<u8>, // debug sphere 2
    pub weapon_hitbox3: Bitflag<u8>, // damipoli

    pub quitout: PointerChain<u8>,
    pub cursor_show: Bitflag<u8>,

    pub gravity: Bitflag<u8>,
    pub display_stable_pos: Bitflag<u8>,
    pub global_position: Position,
    pub stable_position: Position,
    pub chunk_position: Position,
    pub torrent_chunk_position: Position,
    pub animation_speed: PointerChain<f32>,
    pub torrent_animation_speed: PointerChain<f32>,

    pub deathcam: (Bitflag<u8>, PointerChain<u8>),

    // HitIns
    pub hitbox_high: Bitflag<u8>,
    pub hitbox_low: Bitflag<u8>,
    pub hitbox_character: Bitflag<u8>,

    // FieldArea
    pub field_area_direction: Bitflag<u8>,
    pub field_area_altimeter: Bitflag<u8>,
    pub field_area_compass: Bitflag<u8>,

    // GroupMask
    pub show_map: Bitflag<u8>,
    pub show_geom: Vec<Bitflag<u8>>,
    pub show_chr: Bitflag<u8>,

    // Functions
    pub func_item_spawn: usize,
    pub func_item_inject: usize,

    pub base_addresses: BaseAddresses,
}

//
// Position
//

/// Encodes the position vector and two rotation angles.
#[derive(Debug, Clone)]
pub struct Position {
    pub x: PointerChain<f32>,
    pub y: PointerChain<f32>,
    pub z: PointerChain<f32>,
    pub angle1: PointerChain<f32>,
    pub angle2: PointerChain<f32>,
}

impl Position {
    pub fn read(&self) -> Option<[f32; 5]> {
        match (
            self.x.read(),
            self.y.read(),
            self.z.read(),
            self.angle1.read(),
            self.angle2.read(),
        ) {
            (Some(x), Some(y), Some(z), Some(r1), Some(r2)) => Some([x, y, z, r1, r2]),
            _ => None,
        }
    }

    pub fn write(&self, [x, y, z, r1, r2]: [f32; 5]) {
        self.x.write(x);
        self.y.write(y);
        self.z.write(z);
        self.angle1.write(r1);
        self.angle2.write(r2);
    }
}

//
// Character stats
//

#[derive(Debug, Clone)]
#[repr(C)]
pub struct CharacterStats {
    pub vigor: i32,         // 3C
    pub mind: i32,          // 40
    pub endurance: i32,     // 44
    pub strength: i32,      // 48
    pub dexterity: i32,     // 4C
    pub intelligence: i32,  // 50
    pub faith: i32,         // 54
    pub arcane: i32,        // 58
    pub pad1: [u32; 2],     // 60, 64
    pub level: i32,         // 68
    pub runes: i32,         // 6C
    pub runes_tot: i32,     // 70
}

impl Display for CharacterStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "CharacterStats {{ }}")
    }
}

impl Pointers {
    pub fn new() -> Self {
        let base_module_address = unsafe { GetModuleHandleA(PCSTR(null_mut())) }.0 as usize;
        // let base_addresses = match *crate::version::VERSION {
        //     Version::V1_02_0 => base_addresses::BASE_ADDRESSES_1_02_0,
        //     Version::V1_02_1 => base_addresses::BASE_ADDRESSES_1_02_1,
        //     Version::V1_02_2 => base_addresses::BASE_ADDRESSES_1_02_2,
        //     Version::V1_02_3 => base_addresses::BASE_ADDRESSES_1_02_3,
        //     Version::V1_03_0 => base_addresses::BASE_ADDRESSES_1_03_0,
        //     Version::V1_03_1 => base_addresses::BASE_ADDRESSES_1_03_1,
        //     Version::V1_03_2 => base_addresses::BASE_ADDRESSES_1_03_2,
        //     Version::V1_04_0 => base_addresses::BASE_ADDRESSES_1_04_0,
        // }
        let base_addresses = BaseAddresses::from(*crate::version::VERSION)
            .with_module_base_addr(base_module_address);

        let BaseAddresses {
            chr_dbg_flags,
            cs_menu_man_imp,
            damage_ctrl,
            field_area,
            game_data_man,
            group_mask,
            hit_ins_hitbox_offset,
            world_chr_man,
            func_item_spawn,
            func_item_inject,
            ..
        } = base_addresses;

        // Special cases

        let global_position_offset = {
            use Version::*;
            match *crate::version::VERSION {
                V1_02_0 | V1_02_1 | V1_02_2 | V1_02_3 | V1_03_0 | V1_03_1 | V1_03_2 => 0x6b8,
                V1_04_0 | V1_04_1 => 0x6b0,
            }
        };

        Self {
            one_shot: bitflag!(0b1; chr_dbg_flags + 0x3),
            no_damage: bitflag!(0b1; chr_dbg_flags + 0xC),
            no_dead: bitflag!(0b1; chr_dbg_flags + 0x1),
            no_hit: bitflag!(0b1; chr_dbg_flags + 0xD),
            no_goods_consume: bitflag!(0b1; chr_dbg_flags + 0x4),
            no_stamina_consume: bitflag!(0b1; chr_dbg_flags + 0x5),
            no_fp_consume: bitflag!(0b1; chr_dbg_flags + 0x6),
            no_arrows_consume: bitflag!(0b1; chr_dbg_flags + 0x7),
            no_attack: bitflag!(0b1; chr_dbg_flags + 0xE),
            no_move: bitflag!(0b1; chr_dbg_flags + 0xF),
            no_update_ai: bitflag!(0b1; chr_dbg_flags + 0x10),
            no_ashes_of_war_fp_consume: bitflag!(0b1; chr_dbg_flags + 0x12),

            all_no_dead: bitflag!(0b1; chr_dbg_flags + 0xB),

            torrent_no_dead: bitflag!(0b1; chr_dbg_flags + 0x2),
            torrent_gravity: bitflag!(0b1; world_chr_man, 0x18390, 0x18, 0, 0x190, 0x68, 0x1d3),

            // WorldChrMan -> Player
            collision: bitflag!(0b1000; world_chr_man, 0x18468, 0x58, 0xf0),

            // WorldChrMan -> Torrent
            // Torrent ptr: world_chr_man, 0x18390, 0x18, 0
            torrent_collision: bitflag!(0b1000; world_chr_man, 0x18390, 0x18, 0, 0x58, 0xf0),

            character_stats: pointer_chain!(game_data_man, 0x8, 0x3c),
            runes: pointer_chain!(game_data_man, 0x8, 0x6C),
            igt: pointer_chain!(game_data_man, 0xA0),

            quitout: pointer_chain!(cs_menu_man_imp, 0x8, 0x5d),
            cursor_show: bitflag!(0b1; cs_menu_man_imp, 0xAC),
            gravity: bitflag!(0b1; world_chr_man, 0x18468, 0x190, 0x68, 0x1d3),
            display_stable_pos: bitflag!(0b1; world_chr_man, 0x18468, 0x6FD),
            global_position: Position {
                x: pointer_chain!(world_chr_man, 0x18468, global_position_offset),
                y: pointer_chain!(world_chr_man, 0x18468, global_position_offset + 0x4),
                z: pointer_chain!(world_chr_man, 0x18468, global_position_offset + 0x8),
                angle1: pointer_chain!(world_chr_man, 0x18468, 0x6bc),
                angle2: pointer_chain!(world_chr_man, 0x18468, 0x6cc),
            },
            stable_position: Position {
                x: pointer_chain!(world_chr_man, 0x18468, global_position_offset + 0x14),
                y: pointer_chain!(world_chr_man, 0x18468, global_position_offset + 0x18),
                z: pointer_chain!(world_chr_man, 0x18468, global_position_offset + 0x1C),
                angle1: pointer_chain!(world_chr_man, 0x18468, 0x6d8),
                angle2: pointer_chain!(world_chr_man, 0x18468, 0x6e8),
            },
            chunk_position: Position {
                x: pointer_chain!(world_chr_man, 0x18468, 0x190, 0x68, 0x70),
                y: pointer_chain!(world_chr_man, 0x18468, 0x190, 0x68, 0x74),
                z: pointer_chain!(world_chr_man, 0x18468, 0x190, 0x68, 0x78),
                angle1: pointer_chain!(world_chr_man, 0x18468, 0x190, 0x68, 0x54),
                angle2: pointer_chain!(world_chr_man, 0x18468, 0x190, 0x68, 0x64),
            },
            torrent_chunk_position: Position {
                x: pointer_chain!(world_chr_man, 0x18390, 0x18, 0x0, 0x190, 0x68, 0x70),
                y: pointer_chain!(world_chr_man, 0x18390, 0x18, 0x0, 0x190, 0x68, 0x74),
                z: pointer_chain!(world_chr_man, 0x18390, 0x18, 0x0, 0x190, 0x68, 0x78),
                angle1: pointer_chain!(world_chr_man, 0x18390, 0x18, 0x0, 0x190, 0x68, 0x54),
                angle2: pointer_chain!(world_chr_man, 0x18390, 0x18, 0x0, 0x190, 0x68, 0x64),
            },
            animation_speed: pointer_chain!(world_chr_man, 0xB658, 0, 0x190, 0x28, 0x17C8),
            torrent_animation_speed: pointer_chain!(
                world_chr_man,
                0x18390,
                0x18,
                0,
                0x190,
                0x28,
                0x17C8
            ),

            deathcam: (
                bitflag!(0b100; world_chr_man, 0x18468, 0x1c8),
                pointer_chain!(field_area, 0x98, 0x7c),
            ),

            field_area_direction: bitflag!(0b1; field_area + 0x9),
            field_area_altimeter: bitflag!(0b1; field_area + 0xA),
            field_area_compass: bitflag!(0b1; field_area + 0xB),
            weapon_hitbox1: bitflag!(0b1; damage_ctrl, 0xA0),
            weapon_hitbox2: bitflag!(0b1; damage_ctrl, 0xA1),
            weapon_hitbox3: bitflag!(0b1; damage_ctrl, 0xA4),
            hitbox_high: bitflag!(0b1; hit_ins_hitbox_offset),
            hitbox_low: bitflag!(0b1; hit_ins_hitbox_offset + 0x1),
            hitbox_character: bitflag!(0b1; hit_ins_hitbox_offset + 0x3),
            show_map: bitflag!(0b1; group_mask + 0x2),
            show_geom: vec![
                bitflag!(0b1; group_mask + 0x3),
                bitflag!(0b1; group_mask + 0x4),
                bitflag!(0b1; group_mask + 0x5),
                bitflag!(0b1; group_mask + 0x6),
                bitflag!(0b1; group_mask + 0x7),
                bitflag!(0b1; group_mask + 0x8),
                bitflag!(0b1; group_mask + 0x9),
                bitflag!(0b1; group_mask + 0xA),
                bitflag!(0b1; group_mask + 0xB),
                bitflag!(0b1; group_mask + 0xC),
                bitflag!(0b1; group_mask + 0xD),
                bitflag!(0b1; group_mask + 0xF),  // VFX
                bitflag!(0b1; group_mask + 0x10), // Cutscene
                bitflag!(0b1; group_mask + 0x11), // Unknown
                bitflag!(0b1; group_mask + 0x12), // Grass
            ],
            show_chr: bitflag!(0b1; group_mask + 0xE),

            func_item_spawn,
            func_item_inject,
            base_addresses,
        }
    }
}
