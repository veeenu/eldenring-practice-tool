use std::str::FromStr;

use hudhook::tracing::error;
use hudhook::tracing::metadata::LevelFilter;
use libeldenring::prelude::*;
use practice_tool_core::controller::ControllerCombination;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::Widget;
use serde::Deserialize;

use crate::widgets::character_stats::character_stats_edit;
use crate::widgets::cycle_color::cycle_color;
use crate::widgets::cycle_speed::cycle_speed;
use crate::widgets::deathcam::deathcam;
use crate::widgets::flag::flag_widget;
use crate::widgets::group::group;
use crate::widgets::item_spawn::ItemSpawner;
use crate::widgets::label::label_widget;
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
    #[serde(rename = "radial-menu")]
    pub(crate) radial_menu: Vec<RadialMenu>,
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
    pub(crate) radial_menu_open: Option<ControllerCombination>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct RadialMenu {
    pub key: Key,
    pub label: String,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) enum IndicatorType {
    Igt,
    Position,
    PositionChange,
    PositionDistance,
    GameVersion,
    ImguiDebug,
    Fps,
    FrameCount,
    Animation,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "IndicatorConfig")]
pub(crate) struct Indicator {
    pub(crate) indicator: IndicatorType,
    pub(crate) enabled: bool,
}

impl Indicator {
    fn default_set() -> Vec<Indicator> {
        vec![
            Indicator { indicator: IndicatorType::GameVersion, enabled: true },
            Indicator { indicator: IndicatorType::Igt, enabled: true },
            Indicator { indicator: IndicatorType::Position, enabled: false },
            Indicator { indicator: IndicatorType::PositionChange, enabled: false },
            Indicator { indicator: IndicatorType::PositionDistance, enabled: false },
            Indicator { indicator: IndicatorType::Animation, enabled: false },
            Indicator { indicator: IndicatorType::Fps, enabled: false },
            Indicator { indicator: IndicatorType::FrameCount, enabled: false },
            Indicator { indicator: IndicatorType::ImguiDebug, enabled: false },
        ]
    }
}

#[derive(Debug, Deserialize, Clone)]
struct IndicatorConfig {
    indicator: String,
    enabled: bool,
}

impl TryFrom<IndicatorConfig> for Indicator {
    type Error = String;

    fn try_from(indicator: IndicatorConfig) -> Result<Self, Self::Error> {
        match indicator.indicator.as_str() {
            "igt" => Ok(Indicator { indicator: IndicatorType::Igt, enabled: indicator.enabled }),
            "position" => {
                Ok(Indicator { indicator: IndicatorType::Position, enabled: indicator.enabled })
            },
            "position_change" => Ok(Indicator {
                indicator: IndicatorType::PositionChange,
                enabled: indicator.enabled,
            }),
            "position_distance" => Ok(Indicator {
                indicator: IndicatorType::PositionDistance,
                enabled: indicator.enabled,
            }),
            "animation" => {
                Ok(Indicator { indicator: IndicatorType::Animation, enabled: indicator.enabled })
            },
            "game_version" => {
                Ok(Indicator { indicator: IndicatorType::GameVersion, enabled: indicator.enabled })
            },
            "fps" => Ok(Indicator { indicator: IndicatorType::Fps, enabled: indicator.enabled }),
            "framecount" => {
                Ok(Indicator { indicator: IndicatorType::FrameCount, enabled: indicator.enabled })
            },
            "imgui_debug" => {
                Ok(Indicator { indicator: IndicatorType::ImguiDebug, enabled: indicator.enabled })
            },
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
    Label {
        #[serde(rename = "label")]
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
    CycleColor {
        #[serde(rename = "cycle_color")]
        cycle_color: Vec<i32>,
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
                flag_widget(&flag.label, (*(flag.getter)(chains)).clone_box(), hotkey)
            },
            CfgCommand::MultiFlag { flag, hotkey } => multi_flag(
                &flag.label,
                flag.items.iter().map(|flag| flag(chains).clone_box()).collect(),
                hotkey,
            ),
            CfgCommand::MultiFlagUser { flags, hotkey, label } => multi_flag(
                label.as_str(),
                flags.iter().map(|flag| (*(flag.getter)(chains)).clone_box()).collect(),
                hotkey,
            ),
            CfgCommand::SpecialFlag { flag, hotkey } if flag == "deathcam" => deathcam(
                chains.deathcam.0.clone(),
                chains.deathcam.1.clone(),
                chains.deathcam.2.clone(),
                hotkey,
            ),
            CfgCommand::SpecialFlag { flag, hotkey: _ } => {
                error!("Invalid flag {}", flag);
                return None;
            },
            CfgCommand::Label { label } => label_widget(label.as_str()),
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
            CfgCommand::CycleColor { cycle_color: values, hotkey } => {
                cycle_color(values.as_slice(), chains.mesh_color.clone(), hotkey)
            },
            CfgCommand::CharacterStats { hotkey_open } => character_stats_edit(
                chains.character_stats.clone(),
                chains.character_points.clone(),
                chains.character_blessings.clone(),
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
            CfgCommand::Target { hotkey } => Box::new(Target::new(
                chains.current_target.clone(),
                chains.chunk_position.clone(),
                hotkey.into_option(),
            )),
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
                radial_menu_open: ControllerCombination::try_from("l3+r3").ok(),
            },
            radial_menu: Vec::new(),
            commands: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
struct FlagSpec {
    label: String,
    getter: fn(&Pointers) -> Box<dyn FlagToggler>,
}

impl std::fmt::Debug for FlagSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlagSpec {{ label: {:?} }}", self.label)
    }
}

impl FlagSpec {
    fn new(label: &str, getter: fn(&Pointers) -> Box<dyn FlagToggler>) -> FlagSpec {
        FlagSpec { label: label.to_string(), getter }
    }
}

impl TryFrom<String> for FlagSpec {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        macro_rules! flag_spec {
            ($x:expr, [ $( ($flag_name:ident, $flag_label:expr), )* ]) => {
                match $x {
                    $(stringify!($flag_name) => Ok(FlagSpec::new($flag_label, |c| Box::new(c.$flag_name.clone()))),)*
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
            (no_trigger_event, "No trigger events"),
            (runearc, "Rune Arc"),
            (gravity, "No Gravity"),
            (torrent_gravity, "No Gravity (Torrent)"),
            (collision, "No Collision"),
            (torrent_collision, "No Collision (Torrent)"),
            (action_freeze, "Action freeze"),
            (display_stable_pos, "Show stable pos"),
            (weapon_hitbox1, "Weapon hitbox #1"),
            (weapon_hitbox2, "Weapon hitbox #2"),
            (weapon_hitbox3, "Weapon hitbox #3"),
            (hitbox_high, "High world hitbox"),
            (hitbox_low, "Low world hitbox"),
            (hitbox_f, "Walls hitbox"),
            (hitbox_character, "Character hitbox"),
            (hitbox_event, "Event hitbox"),
            (poise_view, "Poise View"),
            (sound_view, "Sound View"),
            (all_targeting_view, "Targeting View"),
            (field_area_direction, "Direction HUD"),
            (field_area_altimeter, "Altimeter HUD"),
            (field_area_compass, "Compass HUD"),
            // (show_map, "Show/hide map"),
            (show_chr, "Show/hide character"),
            (show_all_map_layers, "Show all map layers"),
            (show_all_graces, "Show all graces"),
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
