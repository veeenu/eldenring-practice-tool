use libeldenring::prelude::*;

use std::str::FromStr;

use log::{error, LevelFilter};
use serde::Deserialize;

use crate::util;
use crate::util::KeyState;
use crate::widgets::cycle_speed::CycleSpeed;
use crate::widgets::deathcam::Deathcam;
use crate::widgets::flag::Flag;
use crate::widgets::item_spawn::ItemSpawner;
use crate::widgets::multiflag::MultiFlag;
use crate::widgets::nudge_pos::NudgePosition;
use crate::widgets::position::SavePosition;
use crate::widgets::quitout::Quitout;
use crate::widgets::runes::Runes;
use crate::widgets::savefile_manager::SavefileManager;
use crate::widgets::Widget;

#[cfg_attr(test, derive(Debug))]
#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) settings: Settings,
    commands: Vec<CfgCommand>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Settings {
    pub(crate) log_level: LevelFilterSerde,
    pub(crate) display: KeyState,
}

#[cfg_attr(test, derive(Debug))]
#[derive(Deserialize)]
#[serde(untagged)]
enum CfgCommand {
    SavefileManager {
        #[serde(rename = "savefile_manager")]
        hotkey_load: KeyState,
        hotkey_back: KeyState,
        hotkey_close: KeyState,
    },
    ItemSpawner {
        #[serde(rename = "item_spawner")]
        hotkey_load: KeyState,
        hotkey_close: KeyState,
    },
    Flag {
        flag: FlagSpec,
        hotkey: Option<KeyState>,
    },
    MultiFlag {
        flag: MultiFlagSpec,
        hotkey: Option<KeyState>,
    },
    SpecialFlag {
        flag: String,
        hotkey: Option<KeyState>,
    },
    MultiFlagUser {
        flags: Vec<FlagSpec>,
        hotkey: Option<KeyState>,
        label: String,
    },
    Position {
        #[serde(rename = "position")]
        hotkey: KeyState,
        modifier: KeyState,
    },
    NudgePosition {
        nudge: f32,
        nudge_up: Option<KeyState>,
        nudge_down: Option<KeyState>,
    },
    CycleSpeed {
        #[serde(rename = "cycle_speed")]
        cycle_speed: Vec<f32>,
        hotkey: KeyState,
    },
    Runes {
        #[serde(rename = "runes")]
        amount: u32,
        hotkey: KeyState,
    },
    Quitout {
        #[serde(rename = "quitout")]
        hotkey: KeyState,
    },
}

#[derive(Deserialize, Debug)]
#[serde(try_from = "String")]
pub(crate) struct LevelFilterSerde(log::LevelFilter);

impl LevelFilterSerde {
    pub(crate) fn inner(&self) -> log::LevelFilter {
        self.0
    }
}

impl TryFrom<String> for LevelFilterSerde {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(LevelFilterSerde(
            log::LevelFilter::from_str(&value)
                .map_err(|e| format!("Couldn't parse log level filter: {}", e))?,
        ))
    }
}

impl Config {
    pub(crate) fn parse(cfg: &str) -> Result<Self, String> {
        let de = &mut toml::de::Deserializer::new(cfg);
        serde_path_to_error::deserialize(de)
            .map_err(|e| format!("TOML config error at {}: {}", e.path(), e.inner()))
    }

    pub(crate) fn make_commands(&self, chains: &Pointers) -> Vec<Box<dyn Widget>> {
        self.commands
            .iter()
            .filter_map(|cmd| {
                Some(match cmd {
                    CfgCommand::Flag { flag, hotkey } => Box::new(Flag::new(
                        &flag.label,
                        (flag.getter)(chains).clone(),
                        hotkey.clone(),
                    )) as Box<dyn Widget>,
                    CfgCommand::MultiFlag { flag, hotkey } => Box::new(MultiFlag::new(
                        &flag.label,
                        flag.items.iter().map(|flag| flag(chains).clone()).collect(),
                        hotkey.clone(),
                    ))
                        as Box<dyn Widget>,
                    CfgCommand::MultiFlagUser {
                        flags,
                        hotkey,
                        label,
                    } => Box::new(MultiFlag::new(
                        label,
                        flags
                            .iter()
                            .map(|flag| (flag.getter)(chains).clone())
                            .collect(),
                        hotkey.clone(),
                    )) as Box<dyn Widget>,
                    CfgCommand::SpecialFlag { flag, hotkey } if flag == "deathcam" => {
                        Box::new(Deathcam::new(
                            chains.deathcam.0.clone(),
                            chains.deathcam.1.clone(),
                            hotkey.clone(),
                        ))
                    }
                    CfgCommand::SpecialFlag { flag, hotkey: _ } => {
                        error!("Invalid flag {}", flag);
                        return None;
                    }
                    CfgCommand::SavefileManager {
                        hotkey_load,
                        hotkey_back,
                        hotkey_close,
                    } => SavefileManager::new_widget(
                        hotkey_load.clone(),
                        hotkey_back.clone(),
                        hotkey_close.clone(),
                    ),
                    CfgCommand::ItemSpawner {
                        hotkey_load,
                        hotkey_close,
                    } => Box::new(ItemSpawner::new(
                        chains.func_item_inject,
                        chains.base_addresses.map_item_man,
                        chains.gravity.clone(),
                        hotkey_load.clone(),
                        hotkey_close.clone(),
                    )),
                    CfgCommand::Position { hotkey, modifier } => Box::new(SavePosition::new(
                        chains.global_position.clone(),
                        chains.chunk_position.clone(),
                        chains.torrent_chunk_position.clone(),
                        hotkey.clone(),
                        modifier.clone(),
                    )),
                    CfgCommand::NudgePosition {
                        nudge,
                        nudge_up,
                        nudge_down,
                    } => Box::new(NudgePosition::new(
                        chains.chunk_position.clone(),
                        *nudge,
                        nudge_up.clone(),
                        nudge_down.clone(),
                    )),
                    CfgCommand::CycleSpeed {
                        cycle_speed,
                        hotkey,
                    } => Box::new(CycleSpeed::new(
                        cycle_speed,
                        [
                            chains.animation_speed.clone(),
                            chains.torrent_animation_speed.clone(),
                        ],
                        hotkey.clone(),
                    )),
                    CfgCommand::Runes { amount, hotkey } => {
                        Box::new(Runes::new(*amount, chains.runes.clone(), hotkey.clone()))
                    }
                    CfgCommand::Quitout { hotkey } => {
                        Box::new(Quitout::new(chains.quitout.clone(), hotkey.clone()))
                    }
                })
            })
            .collect()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            settings: Settings {
                log_level: LevelFilterSerde(LevelFilter::Debug),
                display: KeyState::new(util::get_key_code("0").unwrap()),
            },
            commands: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
struct FlagSpec {
    label: String,
    getter: fn(&Pointers) -> &Bitflag<u8>,
}

impl std::fmt::Debug for FlagSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlagSpec {{ label: {:?} }}", self.label)
    }
}

impl FlagSpec {
    fn new(label: &str, getter: fn(&Pointers) -> &Bitflag<u8>) -> FlagSpec {
        FlagSpec {
            label: label.to_string(),
            getter,
        }
    }
}

impl TryFrom<String> for FlagSpec {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        macro_rules! flag_spec {
            ($x:expr, [ $( ($flag_name:ident, $flag_label:expr), )* ]) => {
                match $x {
                    $(stringify!($flag_name) => Ok(FlagSpec::new($flag_label, |c| &c.$flag_name)),)*
                    e => Err(format!("\"{}\" is not a valid flag specifier", e)),
                }
            }
        }
        flag_spec!(
            value.as_str(),
            [
                (one_shot, "One shot"),
                (no_damage, "All no damage"),
                (no_dead, "No death"),
                (no_hit, "No hit"),
                (no_goods_consume, "Inf Consumables"),
                (no_stamina_consume, "Inf Stamina"),
                (no_fp_consume, "Inf Focus"),
                (no_ashes_of_war_fp_consume, "Inf Focus (AoW)"),
                (no_arrows_consume, "Inf arrows"),
                (no_attack, "No attack"),
                (no_move, "No move"),
                (no_update_ai, "No update AI"),
                (gravity, "No Gravity"),
                (torrent_gravity, "No Gravity (Torrent)"),
                (collision, "No Collision"),
                (torrent_collision, "No Collision (Torrent)"),
                (display_stable_pos, "Show stable pos"),
                (weapon_hitbox1, "Weapon hitbox #1"),
                (weapon_hitbox2, "Weapon hitbox #2"),
                (weapon_hitbox3, "Weapon hitbox #3"),
                (hitbox_high, "High world hitbox"),
                (hitbox_low, "Low world hitbox"),
                (hitbox_character, "Character hitbox"),
                (field_area_direction, "Direction HUD"),
                (field_area_altimeter, "Altimeter HUD"),
                (field_area_compass, "Compass HUD"),
                // (show_map, "Show/hide map"),
                (show_chr, "Show/hide character"),
            ]
        )
    }
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
struct MultiFlagSpec {
    label: String,
    items: Vec<fn(&Pointers) -> &Bitflag<u8>>,
}

impl std::fmt::Debug for MultiFlagSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlagSpec {{ label: {:?} }}", self.label)
    }
}

impl MultiFlagSpec {
    fn new(label: &str, items: Vec<fn(&Pointers) -> &Bitflag<u8>>) -> MultiFlagSpec {
        MultiFlagSpec {
            label: label.to_string(),
            items,
        }
    }
}

impl TryFrom<String> for MultiFlagSpec {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "show_map" => Ok(MultiFlagSpec::new(
                "Show/hide map",
                vec![
                    |c| &c.show_geom[0],
                    |c| &c.show_geom[1],
                    |c| &c.show_geom[2],
                    |c| &c.show_geom[3],
                    |c| &c.show_geom[4],
                    |c| &c.show_geom[5],
                    |c| &c.show_geom[6],
                    |c| &c.show_geom[7],
                    |c| &c.show_geom[8],
                    |c| &c.show_geom[9],
                    |c| &c.show_geom[10],
                    |c| &c.show_geom[11],
                    |c| &c.show_geom[12],
                    |c| &c.show_geom[13],
                    |c| &c.show_geom[14],
                    |c| &c.show_map,
                ],
            )),
            e => Err(format!("\"{}\" is not a valid multiflag specifier", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_parse() {
        println!(
            "{:?}",
            toml::from_str::<toml::Value>(include_str!("../../jdsd_er_practice_tool.toml"))
        );
        println!(
            "{:?}",
            Config::parse(include_str!("../../jdsd_er_practice_tool.toml"))
        );
    }

    #[test]
    fn test_parse_errors() {
        println!(
            "{:#?}",
            Config::parse(
                r#"commands = [ { boh = 3 } ]
                [settings]
                log_level = "DEBUG"
                "#
            )
        );
    }
}
