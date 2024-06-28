use std::str::FromStr;

use hudhook::tracing::error;
use hudhook::tracing::metadata::LevelFilter;
use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::Widget;
use serde::Deserialize;

use crate::widgets::action_freeze::action_freeze;
use crate::widgets::character_stats::character_stats_edit;
use crate::widgets::cycle_speed::cycle_speed;
use crate::widgets::deathcam::deathcam;
use crate::widgets::flag::flag_widget;
use crate::widgets::group::group;
use crate::widgets::item_spawn::ItemSpawner;
use crate::widgets::multiflag::multi_flag;
use crate::widgets::nudge_pos::nudge_position;
use crate::widgets::position::save_position;
use crate::widgets::quitout::quitout;
use crate::widgets::runes::runes;
use crate::widgets::savefile_manager::savefile_manager;
use crate::widgets::target::Target;
use crate::widgets::warp::Warp;

#[cfg_attr(test, derive(Debug))]
#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) settings: Settings,
    commands: Vec<CfgCommand>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Settings {
    pub(crate) log_level: LevelFilterSerde,
    pub(crate) display: Key,
    pub(crate) hide: Option<Key>,
    #[serde(default)]
    pub(crate) dxgi_debug: bool,
    #[serde(default)]
    pub(crate) show_console: bool,
    #[serde(default)]
    pub(crate) disable_update_prompt: bool,
    #[serde(default = "Indicator::default_set")]
    pub(crate) indicators: Vec<Indicator>,
}

#[derive(Deserialize, Copy, Clone, Debug)]
#[serde(try_from = "String")]
pub(crate) enum Indicator {
    Igt,
    Position,
    GameVersion,
    ImguiDebug,
}

impl Indicator {
    fn default_set() -> Vec<Indicator> {
        vec![Indicator::GameVersion, Indicator::Position, Indicator::Igt]
    }
}

impl TryFrom<String> for Indicator {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "igt" => Ok(Indicator::Igt),
            "position" => Ok(Indicator::Position),
            "game_version" => Ok(Indicator::GameVersion),
            "imgui_debug" => Ok(Indicator::ImguiDebug),
            value => Err(format!("Unrecognized indicator: {value}")),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum PlaceholderOption<T> {
    Data(T),
    #[allow(dead_code)]
    Placeholder(bool),
}

impl<T> PlaceholderOption<T> {
    fn into_option(self) -> Option<T> {
        match self {
            PlaceholderOption::Data(d) => Some(d),
            PlaceholderOption::Placeholder(_) => None,
        }
    }
}

#[cfg_attr(test, derive(Debug))]
#[derive(Deserialize)]
#[serde(untagged)]
enum CfgCommand {
    SavefileManager {
        #[serde(rename = "savefile_manager")]
        hotkey_load: PlaceholderOption<Key>,
    },
    ItemSpawner {
        #[serde(rename = "item_spawner")]
        hotkey_load: PlaceholderOption<Key>,
    },
    Flag {
        flag: FlagSpec,
        hotkey: Option<Key>,
    },
    MultiFlag {
        flag: MultiFlagSpec,
        hotkey: Option<Key>,
    },
    SpecialFlag {
        flag: String,
        hotkey: Option<Key>,
    },
    MultiFlagUser {
        flags: Vec<FlagSpec>,
        hotkey: Option<Key>,
        label: String,
    },
    Position {
        position: PlaceholderOption<Key>,
        save: Option<Key>,
    },
    NudgePosition {
        nudge: f32,
        nudge_up: Option<Key>,
        nudge_down: Option<Key>,
    },
    CycleSpeed {
        #[serde(rename = "cycle_speed")]
        cycle_speed: Vec<f32>,
        hotkey: Option<Key>,
    },
    CharacterStats {
        #[serde(rename = "character_stats")]
        hotkey_open: PlaceholderOption<Key>,
    },
    Runes {
        #[serde(rename = "runes")]
        amount: u32,
        hotkey: Option<Key>,
    },
    Target {
        #[serde(rename = "target")]
        hotkey: PlaceholderOption<Key>,
    },
    Warp {
        #[serde(rename = "warp")]
        _warp: bool,
    },
    Group {
        #[serde(rename = "group")]
        label: String,
        commands: Vec<CfgCommand>,
    },
    Quitout {
        #[serde(rename = "quitout")]
        hotkey: PlaceholderOption<Key>,
    },
}

impl CfgCommand {
    fn into_widget(self, settings: &Settings, chains: &Pointers) -> Option<Box<dyn Widget>> {
        let widget = match self {
            CfgCommand::Flag { flag, hotkey } => {
                flag_widget(&flag.label, (flag.getter)(chains).clone(), hotkey)
            },
            CfgCommand::MultiFlag { flag, hotkey } => multi_flag(
                &flag.label,
                flag.items.iter().map(|flag| flag(chains).clone()).collect(),
                hotkey,
            ),
            CfgCommand::MultiFlagUser { flags, hotkey, label } => multi_flag(
                label.as_str(),
                flags.iter().map(|flag| (flag.getter)(chains).clone()).collect(),
                hotkey,
            ),
            CfgCommand::SpecialFlag { flag, hotkey } if flag == "deathcam" => deathcam(
                chains.deathcam.0.clone(),
                chains.deathcam.1.clone(),
                chains.deathcam.2.clone(),
                hotkey,
            ),
            CfgCommand::SpecialFlag { flag, hotkey } if flag == "action_freeze" => action_freeze(
                chains.func_dbg_action_force.clone(),
                chains.func_dbg_action_force_state_values,
                hotkey,
            ),
            CfgCommand::SpecialFlag { flag, hotkey: _ } => {
                error!("Invalid flag {}", flag);
                return None;
            },
            CfgCommand::SavefileManager { hotkey_load } => {
                savefile_manager(hotkey_load.into_option(), settings.display)
            },
            CfgCommand::ItemSpawner { hotkey_load } => Box::new(ItemSpawner::new(
                chains.func_item_inject,
                chains.base_addresses.map_item_man,
                chains.gravity.clone(),
                hotkey_load.into_option(),
                settings.display,
            )),
            CfgCommand::Position { position, save } => save_position(
                chains.global_position.clone(),
                chains.chunk_position.clone(),
                chains.torrent_chunk_position.clone(),
                position.into_option(),
                save,
            ),
            CfgCommand::NudgePosition { nudge, nudge_up, nudge_down } => nudge_position(
                chains.global_position.clone(),
                chains.chunk_position.clone(),
                chains.torrent_chunk_position.clone(),
                nudge,
                nudge_up,
                nudge_down,
            ),
            CfgCommand::CycleSpeed { cycle_speed: values, hotkey } => cycle_speed(
                values.as_slice(),
                [chains.animation_speed.clone(), chains.torrent_animation_speed.clone()],
                hotkey,
            ),
            CfgCommand::CharacterStats { hotkey_open } => character_stats_edit(
                chains.character_stats.clone(),
                hotkey_open.into_option(),
                settings.display,
            ),
            CfgCommand::Runes { amount, hotkey } => runes(amount, chains.runes.clone(), hotkey),
            CfgCommand::Warp { .. } => Box::new(Warp::new(
                chains.func_warp,
                chains.warp1.clone(),
                chains.warp2.clone(),
                settings.display,
            )),
            CfgCommand::Target { hotkey } => {
                Box::new(Target::new(chains.current_target.clone(), hotkey.into_option()))
            },
            CfgCommand::Quitout { hotkey } => quitout(chains.quitout.clone(), hotkey.into_option()),
            CfgCommand::Group { label, commands } => group(
                label.as_str(),
                commands.into_iter().filter_map(|c| c.into_widget(settings, chains)).collect(),
                settings.display,
            ),
        };

        Some(widget)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(try_from = "String")]
pub(crate) struct LevelFilterSerde(LevelFilter);

impl LevelFilterSerde {
    pub(crate) fn inner(&self) -> LevelFilter {
        self.0
    }
}

impl TryFrom<String> for LevelFilterSerde {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(LevelFilterSerde(
            LevelFilter::from_str(&value)
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

    pub(crate) fn make_commands(self, chains: &Pointers) -> Vec<Box<dyn Widget>> {
        self.commands.into_iter().filter_map(|c| c.into_widget(&self.settings, chains)).collect()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            settings: Settings {
                log_level: LevelFilterSerde(LevelFilter::DEBUG),
                display: "0".parse().unwrap(),
                hide: "rshift+0".parse().ok(),
                dxgi_debug: false,
                show_console: false,
                indicators: Indicator::default_set(),
                disable_update_prompt: false,
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
        FlagSpec { label: label.to_string(), getter }
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
        flag_spec!(value.as_str(), [
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
            (hitbox_f, "Walls hitbox"),
            (hitbox_character, "Character hitbox"),
            (field_area_direction, "Direction HUD"),
            (field_area_altimeter, "Altimeter HUD"),
            (field_area_compass, "Compass HUD"),
            // (show_map, "Show/hide map"),
            (show_chr, "Show/hide character"),
        ])
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
        MultiFlagSpec { label: label.to_string(), items }
    }
}

impl TryFrom<String> for MultiFlagSpec {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "show_map" => Ok(MultiFlagSpec::new("Show/hide map", vec![
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
                |c| &c.show_geom[if c.show_geom.len() <= 13 { 12 } else { 13 }], // UGLY
                |c| &c.show_geom[if c.show_geom.len() <= 13 { 12 } else { 14 }], // AS
                |c| &c.show_geom[if c.show_geom.len() <= 13 { 12 } else { 15 }], // SIN
            ])),
            e => Err(format!("\"{}\" is not a valid multiflag specifier", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_parse_ok() {
        println!(
            "{:?}",
            toml::from_str::<toml::Value>(include_str!("../../jdsd_er_practice_tool.toml"))
        );
        println!("{:?}", Config::parse(include_str!("../../jdsd_er_practice_tool.toml")));
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
