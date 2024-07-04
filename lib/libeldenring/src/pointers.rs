#![allow(clippy::new_without_default)]

use std::fmt::Display;

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

    pub current_target: PointerChain<u64>,

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

    // CSLuaEventManager
    pub func_warp: usize,
    pub warp1: PointerChain<u64>,
    pub warp2: PointerChain<u64>,

    pub deathcam: (Bitflag<u8>, Bitflag<u8>, PointerChain<u8>),

    // HitIns
    pub hitbox_high: Bitflag<u8>,
    pub hitbox_low: Bitflag<u8>,
    pub hitbox_f: Bitflag<u8>,
    pub hitbox_character: Bitflag<u8>,

    // FieldArea
    pub field_area_direction: Bitflag<u8>,
    pub field_area_altimeter: Bitflag<u8>,
    pub field_area_compass: Bitflag<u8>,

    // GroupMask
    // pub show_map: Bitflag<u8>,
    pub show_geom: Vec<Bitflag<u8>>,
    pub show_chr: Bitflag<u8>,

    // Functions
    pub func_item_spawn: usize,
    pub func_item_inject: usize,
    pub func_dbg_action_force: PointerChain<u8>,
    pub func_dbg_action_force_state_values: (u8, u8),

    pub base_addresses: BaseAddresses,
}

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
    pub map_id: Option<PointerChain<u32>>,
}

impl Position {
    pub fn read(&self) -> Option<[f32; 5]> {
        match (self.x.read(), self.y.read(), self.z.read(), self.angle1.read(), self.angle2.read())
        {
            (Some(x), Some(y), Some(z), Some(r1), Some(r2)) => Some([x, y, z, r1, r2]),
            _ => None,
        }
    }

    pub fn read_map_id(&self) -> Option<u32> {
        self.map_id.as_ref().and_then(|m| m.read())
    }

    pub fn write(&self, [x, y, z, r1, r2]: [f32; 5]) {
        self.x.write(x);
        self.y.write(y);
        self.z.write(z);
        self.angle1.write(r1);
        self.angle2.write(r2);
    }

    pub fn write_map_id(&self, map_id: u32) {
        if let Some(m) = self.map_id.as_ref() {
            m.write(map_id);
        }
    }
}

// Character stats
//

#[derive(Debug, Clone)]
#[repr(C)]
pub struct CharacterStats {
    pub vigor: i32,
    pub mind: i32,
    pub endurance: i32,
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub faith: i32,
    pub arcane: i32,
    pub pad1: [u32; 3],
    pub level: i32,
    pub runes: i32,
    pub runes_tot: i32,
}

impl Display for CharacterStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "CharacterStats {{ }}")
    }
}

impl Pointers {
    pub fn new() -> Self {
        let base_module_address = unsafe { GetModuleHandleA(None).unwrap() }.0 as usize;
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
            lua_warp,
            cs_lua_event_manager,
            current_target,
            ..
        } = base_addresses;

        // Special cases

        let version = *crate::version::VERSION;
        use Version::*;

        let map_id_offset = {
            match version {
                V1_02_0 | V1_02_1 | V1_02_2 | V1_02_3 | V1_03_0 | V1_03_1 | V1_03_2 => 0x6c8,
                V1_04_0 | V1_04_1 | V1_05_0 | V1_06_0 | V1_07_0 => 0x6c0,
                V1_08_0 | V1_08_1 | V1_09_0 | V1_09_1 | V2_00_0 | V2_00_1 | V2_02_0 | V2_02_3 => {
                    0x6d0
                },
            }
        };

        let global_position_offset = {
            match version {
                V1_02_0 | V1_02_1 | V1_02_2 | V1_02_3 | V1_03_0 | V1_03_1 | V1_03_2 => 0x6b8,
                V1_04_0 | V1_04_1 | V1_05_0 | V1_06_0 | V1_07_0 => 0x6b0,
                V1_08_0 | V1_08_1 | V1_09_0 | V1_09_1 | V2_00_0 | V2_00_1 | V2_02_0 | V2_02_3 => {
                    0x6c0
                },
            }
        };

        let group_mask = match version {
            V1_02_0 | V1_02_1 | V1_02_2 | V1_02_3 | V1_03_0 | V1_03_1 | V1_03_2 | V1_04_0
            | V1_04_1 => group_mask,
            V1_05_0 => group_mask - 8,
            V1_06_0 | V1_07_0 | V1_08_0 | V1_08_1 | V1_09_0 | V1_09_1 | V2_00_0 | V2_00_1
            | V2_02_0 | V2_02_3 => group_mask,
        };

        let show_geom = match version {
            V1_02_0 | V1_02_1 | V1_02_2 | V1_02_3 | V1_03_0 | V1_03_1 | V1_03_2 | V1_04_0
            | V1_04_1 | V1_06_0 | V1_07_0 | V1_08_0 | V1_08_1 | V1_09_0 | V1_09_1 | V2_00_0
            | V2_00_1 | V2_02_0 | V2_02_3 => {
                vec![
                    bitflag!(0b1; group_mask + 2),
                    bitflag!(0b1; group_mask + 3),
                    bitflag!(0b1; group_mask + 4),
                    bitflag!(0b1; group_mask + 5),
                    bitflag!(0b1; group_mask + 6),
                    bitflag!(0b1; group_mask + 7),
                    bitflag!(0b1; group_mask + 8),
                    bitflag!(0b1; group_mask),
                    bitflag!(0b1; group_mask + 0xa),
                    bitflag!(0b1; group_mask + 0xb),
                    bitflag!(0b1; group_mask + 0xc),
                    bitflag!(0b1; group_mask + 0xd),
                    bitflag!(0b1; group_mask + 0xf),
                    bitflag!(0b1; group_mask + 0x10),
                    bitflag!(0b1; group_mask + 0x11),
                    bitflag!(0b1; group_mask + 0x12),
                ]
            },
            V1_05_0 => vec![
                bitflag!(0b1; group_mask),
                bitflag!(0b1; group_mask + 1),
                bitflag!(0b1; group_mask + 2),
                bitflag!(0b1; group_mask + 3),
                bitflag!(0b1; group_mask + 5),
                bitflag!(0b1; group_mask + 6),
                bitflag!(0b1; group_mask + 8),
                bitflag!(0b1; group_mask + 0xa),
                bitflag!(0b1; group_mask + 0xb),
                bitflag!(0b1; group_mask + 0xc),
                bitflag!(0b1; group_mask + 0xd),
                bitflag!(0b1; group_mask + 0xe),
                bitflag!(0b1; group_mask + 0xf),
            ],
        };

        let show_chr = match version {
            V1_02_0 | V1_02_1 | V1_02_2 | V1_02_3 | V1_03_0 | V1_03_1 | V1_03_2 | V1_04_0
            | V1_04_1 | V1_06_0 | V1_07_0 | V1_08_0 | V1_08_1 | V1_09_0 | V1_09_1 | V2_00_0
            | V2_00_1 | V2_02_0 | V2_02_3 => {
                bitflag!(0b1; group_mask + 0xe)
            },
            V1_05_0 => bitflag!(0b1; group_mask + 4),
        };

        let player_ins = match version {
            V1_02_0 | V1_02_1 | V1_02_2 | V1_02_3 | V1_03_0 | V1_03_1 | V1_03_2 | V1_04_0
            | V1_04_1 | V1_05_0 | V1_06_0 => 0x18468,
            V1_07_0 | V1_08_0 | V1_08_1 | V1_09_0 | V1_09_1 | V2_00_0 | V2_00_1 | V2_02_0
            | V2_02_3 => 0x1E508,
        };

        let torrent_enemy_ins = match version {
            V1_02_0 | V1_02_1 | V1_02_2 | V1_02_3 | V1_03_0 | V1_03_1 | V1_03_2 | V1_04_0
            | V1_04_1 | V1_05_0 => 0x18390,
            V1_06_0 => 0x18378,
            V1_07_0 => 0x1E1A0,
            V1_08_0 | V1_08_1 | V1_09_0 | V1_09_1 | V2_00_0 | V2_00_1 => 0x1e1b8,
            V2_02_0 | V2_02_3 => 0x1cc90,
        };

        // TODO 1.08.x
        // - show stable position is broken
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
            torrent_gravity: bitflag!(0b1; world_chr_man, torrent_enemy_ins, 0x18, 0, 0x190, 0x68, 0x1d3),

            // WorldChrMan -> Player
            collision: bitflag!(0b1000; world_chr_man, player_ins, 0x58, 0xf0),

            // WorldChrMan -> Torrent
            torrent_collision: bitflag!(0b1000; world_chr_man, torrent_enemy_ins, 0x18, 0, 0x58, 0xf0),

            character_stats: pointer_chain!(game_data_man, 0x8, 0x3c),
            runes: pointer_chain!(game_data_man, 0x8, 0x6C),
            igt: pointer_chain!(game_data_man, 0xA0),

            quitout: pointer_chain!(cs_menu_man_imp, 0x8, 0x5d),
            cursor_show: bitflag!(0b1; cs_menu_man_imp, 0xAC),
            gravity: bitflag!(0b1; world_chr_man, player_ins, 0x190, 0x68, 0x1d3),
            display_stable_pos: bitflag!(0b1; world_chr_man, player_ins,
                match version {
                    V1_02_0 | V1_02_1 | V1_02_2 | V1_02_3 | V1_03_0 | V1_03_1 | V1_03_2 => 0x6FD,
                    V1_04_0 | V1_04_1 | V1_05_0 | V1_06_0 | V1_07_0 => 0x6F5,
                    V1_08_0 | V1_08_1 | V1_09_0 | V1_09_1  | V2_00_0| V2_00_1 | V2_02_0 | V2_02_3 => 0x735
                }
            ),
            global_position: Position {
                x: pointer_chain!(world_chr_man, player_ins, global_position_offset),
                y: pointer_chain!(world_chr_man, player_ins, global_position_offset + 0x4),
                z: pointer_chain!(world_chr_man, player_ins, global_position_offset + 0x8),
                angle1: pointer_chain!(world_chr_man, player_ins, 0x6bc),
                angle2: pointer_chain!(world_chr_man, player_ins, 0x6cc),
                map_id: Some(pointer_chain!(world_chr_man, player_ins, map_id_offset)),
            },
            stable_position: Position {
                x: pointer_chain!(world_chr_man, player_ins, global_position_offset + 0x14),
                y: pointer_chain!(world_chr_man, player_ins, global_position_offset + 0x18),
                z: pointer_chain!(world_chr_man, player_ins, global_position_offset + 0x1C),
                angle1: pointer_chain!(world_chr_man, player_ins, 0x6d8),
                angle2: pointer_chain!(world_chr_man, player_ins, 0x6e8),
                map_id: None,
            },
            chunk_position: Position {
                x: pointer_chain!(world_chr_man, player_ins, 0x190, 0x68, 0x70),
                y: pointer_chain!(world_chr_man, player_ins, 0x190, 0x68, 0x74),
                z: pointer_chain!(world_chr_man, player_ins, 0x190, 0x68, 0x78),
                angle1: pointer_chain!(world_chr_man, player_ins, 0x190, 0x68, 0x54),
                angle2: pointer_chain!(world_chr_man, player_ins, 0x190, 0x68, 0x64),
                map_id: Some(pointer_chain!(world_chr_man, player_ins, map_id_offset)),
            },
            torrent_chunk_position: Position {
                x: pointer_chain!(world_chr_man, torrent_enemy_ins, 0x18, 0x0, 0x190, 0x68, 0x70),
                y: pointer_chain!(world_chr_man, torrent_enemy_ins, 0x18, 0x0, 0x190, 0x68, 0x74),
                z: pointer_chain!(world_chr_man, torrent_enemy_ins, 0x18, 0x0, 0x190, 0x68, 0x78),
                angle1: pointer_chain!(
                    world_chr_man,
                    torrent_enemy_ins,
                    0x18,
                    0x0,
                    0x190,
                    0x68,
                    0x54
                ),
                angle2: pointer_chain!(
                    world_chr_man,
                    torrent_enemy_ins,
                    0x18,
                    0x0,
                    0x190,
                    0x68,
                    0x64
                ),
                map_id: Some(pointer_chain!(
                    world_chr_man,
                    torrent_enemy_ins,
                    0x18,
                    0x0,
                    map_id_offset
                )),
            },
            animation_speed: pointer_chain!(world_chr_man, player_ins, 0x190, 0x28, 0x17C8),
            torrent_animation_speed: pointer_chain!(
                world_chr_man,
                torrent_enemy_ins,
                0x18,
                0,
                0x190,
                0x28,
                0x17C8
            ),

            deathcam: (
                bitflag!(0b100; world_chr_man, player_ins, 0x1c8),
                bitflag!(0b100; world_chr_man, torrent_enemy_ins, 0x18, 0, 0x1c8),
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
            hitbox_f: bitflag!(0b1; hit_ins_hitbox_offset + 0x4),
            hitbox_character: bitflag!(0b1; hit_ins_hitbox_offset + 0x3),
            show_geom,
            show_chr,

            func_warp: lua_warp + 2,
            warp1: pointer_chain!(cs_lua_event_manager, 0x18),
            warp2: pointer_chain!(cs_lua_event_manager, 0x08),

            func_item_spawn,
            func_item_inject,
            func_dbg_action_force: pointer_chain!(base_addresses.func_dbg_action_force + 7),
            func_dbg_action_force_state_values: match version {
                V1_02_0 | V1_02_1 | V1_02_2 | V1_02_3 | V1_03_0 | V1_03_1 | V1_03_2 | V1_04_0
                | V1_04_1 | V1_05_0 | V1_06_0 | V1_07_0 => (0xB1, 0xB2),
                V1_08_0 | V1_08_1 | V1_09_0 | V1_09_1 | V2_00_0 | V2_00_1 | V2_02_0 | V2_02_3 => {
                    (0xC1, 0xC2)
                },
            },
            current_target: pointer_chain!(current_target),
            base_addresses,
        }
    }
}
