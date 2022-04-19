use macro_param::*;

#[derive(ParamStruct)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct EquipParamGoods {
    pub ref_id1: i32,
    pub sfx_variation_id: i32,
    pub weight: f32,
    pub fragment_num: i32,
    pub sell_value: i32,
    pub replace_item_id: i32,
    pub behavior_id: i32,
    pub sort_id: i32,
    pub qwc_id: i32,
    pub yes_no_dialog_message_id: i32,
    pub magic_id: i32,
    pub icon_id: i16,
    pub model_id: i16,
    pub shop_lv: i16,
    pub comp_trophy_sed_id: i16,
    pub trophy_seq_id: i16,
    pub max_num: i16,
    pub consume_hero_point: u8,
    pub over_dexterity: u8,
    pub goods_type: u8,
    pub ref_category: u8,
    pub sp_effect_category: u8,
    pub goods_category: u8,
    pub goods_use_anim: i8,
    pub opme_menu_type: u8,
    pub use_limit_category: u8,
    pub replace_category: u8,
    #[bitflag(vowType0, 0)]
    #[bitflag(vowType1, 1)]
    #[bitflag(vowType2, 2)]
    #[bitflag(vowType3, 3)]
    #[bitflag(vowType4, 4)]
    #[bitflag(vowType5, 5)]
    #[bitflag(vowType6, 6)]
    #[bitflag(vowType7, 7)]
    pub bitfield0: u8,
    #[bitflag(vowType8, 0)]
    #[bitflag(vowType9, 1)]
    #[bitflag(vowType10, 2)]
    #[bitflag(vowType11, 3)]
    #[bitflag(vowType12, 4)]
    #[bitflag(vowType13, 5)]
    #[bitflag(vowType14, 6)]
    #[bitflag(vowType15, 7)]
    pub bitfield1: u8,
    #[bitflag(enable_live, 0)]
    #[bitflag(enable_gray, 1)]
    #[bitflag(enable_white, 2)]
    #[bitflag(enable_black, 3)]
    #[bitflag(enable_multi, 4)]
    #[bitflag(disable_offline, 5)]
    #[bitflag(isEquip, 6)]
    #[bitflag(isConsume, 7)]
    pub bitfield2: u8,
    #[bitflag(isAutoEquip, 0)]
    #[bitflag(isEstablishment, 1)]
    #[bitflag(isOnlyOne, 2)]
    #[bitflag(isDrop, 3)]
    #[bitflag(isDeposit, 4)]
    #[bitflag(isDisableHand, 5)]
    #[bitflag(isTravelItem, 6)]
    #[bitflag(isSuppleItem, 7)]
    pub bitfield3: u8,
    #[bitflag(isFullSuppleItem, 0)]
    #[bitflag(isEnhance, 1)]
    #[bitflag(isFixItem, 2)]
    #[bitflag(disableMutliDropShare, 3)]
    #[bitflag(disableUseAtColiseum, 4)]
    #[bitflag(disableUseAtOutofColiseum, 5)]
    #[bitflag(useBulletMaxNum, 6)]
    #[bitflag(useHpCureMaxNum, 7)]
    pub bitfield4: u8,
    #[bitflag(isAutoReplenish, 0)]
    #[bitflag(canMultiUse, 1)]
    #[bitflag(isGuestDrop, 2)]
    #[bitflag(isEnchantLeftHand, 3)]
    #[bitflag(isApplySpecialEffect, 4)]
    #[bitflag(Unk1, 5)]
    #[bitflag(Unk2, 6)]
    #[bitflag(Unk3, 7)]
    pub bitfield5: u8,
    pub ref_id2: i32,
    pub reinforce_param_weapon: i32,
    pub vagrant_item_lot_id: i32,
    pub vagrant_bonus_ene_drop_item_lot_id: i32,
    pub vagrant_itemene_drop_item_lot_id: i32,
    pub ref_virtual_wep_id: i32,
    pub replace_item_id_by_sp_effect: i32,
    pub replace_trigger_sp_effect_id: i32,
    #[bitflag(isLoadOfCinder, 0)]
    #[bitflag(isPlayRegion1, 1)]
    #[bitflag(isLadder, 2)]
    #[bitflag(isMultiPlay, 3)]
    #[bitflag(useSelected, 4)]
    #[bitflag(Unk4, 5)]
    #[bitflag(isPlayRegion2, 6)]
    #[bitflag(isNetPenalized, 7)]
    pub bitfield6: u8,
    pub supple_item_type: u8,
    pub menu_adhoc_type: u8,
    pub drop: u8,
    pub max_rep_num: i16,
    pub invade_type: u8,
    pub pad1: [u8; 1],
    pub shop_id: i32,
    pub fp_consume: i16,
    pub use_limit_category2: i16,
    pub pad2: [u8; 8],
}

fn main() {}
