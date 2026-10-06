use std::path::PathBuf;

use hudhook::tracing::error;
use libeldenring::prelude::*;
use practice_tool_core::config::{parse_toml, LevelFilterSerde, PlaceholderOption, RadialMenu};
use practice_tool_core::config_editor::{ConfigSchema, Field, Kind};
use practice_tool_core::controller::ControllerCombination;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::input_viewer::InputViewer;
use practice_tool_core::widgets::Widget;
use practice_tool_memedit::widgets::{flag_widget, multi_flag};
use serde::Deserialize;

use crate::util::get_dll_path;
use crate::widgets::character_stats::character_stats_edit;
use crate::widgets::cycle_color::cycle_color;
use crate::widgets::cycle_speed::cycle_speed;
use crate::widgets::deathcam::deathcam;
use crate::widgets::group::group;
use crate::widgets::item_spawn::ItemSpawner;
use crate::widgets::label::label_widget;
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

#[derive(Debug, Deserialize, Clone, Copy)]
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

/// Indicator specifiers, their types, and whether they're enabled by default.
pub(crate) const INDICATORS: &[(&str, IndicatorType, bool)] = &[
    ("game_version", IndicatorType::GameVersion, true),
    ("igt", IndicatorType::Igt, true),
    ("position", IndicatorType::Position, false),
    ("position_change", IndicatorType::PositionChange, false),
    ("position_distance", IndicatorType::PositionDistance, false),
    ("animation", IndicatorType::Animation, false),
    ("fps", IndicatorType::Fps, false),
    ("framecount", IndicatorType::FrameCount, false),
    ("imgui_debug", IndicatorType::ImguiDebug, false),
];

impl Indicator {
    fn default_set() -> Vec<Indicator> {
        INDICATORS.iter().map(|&(_, indicator, enabled)| Indicator { indicator, enabled }).collect()
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
        INDICATORS
            .iter()
            .find(|(id, ..)| *id == indicator.indicator)
            .map(|&(_, kind, _)| Indicator { indicator: kind, enabled: indicator.enabled })
            .ok_or_else(|| format!("Unrecognized indicator: {}", indicator.indicator))
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
    InputViewer {
        #[serde(rename = "input_viewer")]
        hotkey: PlaceholderOption<Key>,
        #[serde(default = "default_input_viewer_seconds")]
        seconds: usize,
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
    fn into_widget(
        self,
        settings: &Settings,
        chains: &'static Pointers,
    ) -> Option<Box<dyn Widget>> {
        let widget = match self {
            CfgCommand::Flag { flag, hotkey } => {
                flag_widget(&flag.label, (flag.getter)(chains), hotkey)
            },
            CfgCommand::MultiFlag { flag, hotkey } => multi_flag(
                &flag.label,
                flag.items.iter().map(|flag| flag(chains)).collect(),
                hotkey,
            ),
            CfgCommand::MultiFlagUser { flags, hotkey, label } => multi_flag(
                label.as_str(),
                flags.iter().map(|flag| (flag.getter)(chains)).collect(),
                hotkey,
            ),
            CfgCommand::SpecialFlag { flag, hotkey } if flag == "deathcam" => {
                deathcam(&chains.deathcam.0, &chains.deathcam.1, &chains.deathcam.2, hotkey)
            },
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
                &chains.gravity,
                hotkey_load.into_option(),
                settings.display,
            )),
            CfgCommand::Position { position, save } => save_position(
                &chains.global_position,
                &chains.chunk_position,
                &chains.torrent_chunk_position,
                position.into_option(),
                save,
            ),
            CfgCommand::NudgePosition { nudge, nudge_up, nudge_down } => nudge_position(
                &chains.global_position,
                &chains.chunk_position,
                &chains.torrent_chunk_position,
                nudge,
                nudge_up,
                nudge_down,
            ),
            CfgCommand::CycleSpeed { cycle_speed: values, hotkey } => cycle_speed(
                values.as_slice(),
                [&chains.animation_speed, &chains.torrent_animation_speed],
                hotkey,
            ),
            CfgCommand::CycleColor { cycle_color: values, hotkey } => {
                cycle_color(values.as_slice(), &chains.mesh_color, hotkey)
            },
            CfgCommand::CharacterStats { hotkey_open } => character_stats_edit(
                &chains.character_stats,
                &chains.character_points,
                chains.character_blessings.as_ref(),
                hotkey_open.into_option(),
                settings.display,
            ),
            CfgCommand::Runes { amount, hotkey } => runes(amount, &chains.runes, hotkey),
            CfgCommand::Warp { .. } => Box::new(Warp::new(
                chains.func_warp,
                &chains.warp1,
                &chains.warp2,
                settings.display,
            )),
            CfgCommand::Target { hotkey } => Box::new(Target::new(
                &chains.current_target,
                &chains.chunk_position,
                hotkey.into_option(),
            )),
            CfgCommand::InputViewer { hotkey, seconds } => {
                Box::new(InputViewer::new(seconds, hotkey.into_option()))
            },
            CfgCommand::Quitout { hotkey } => quitout(&chains.quitout, hotkey.into_option()),
            CfgCommand::Group { label, commands } => group(
                label.as_str(),
                commands.into_iter().filter_map(|c| c.into_widget(settings, chains)).collect(),
                settings.display,
            ),
        };

        Some(widget)
    }
}

fn default_input_viewer_seconds() -> usize {
    5
}

impl Config {
    pub(crate) fn parse(cfg: &str) -> Result<Self, String> {
        parse_toml(cfg)
    }

    pub(crate) fn make_commands(self, chains: &'static Pointers) -> Vec<Box<dyn Widget>> {
        self.commands.into_iter().filter_map(|c| c.into_widget(&self.settings, chains)).collect()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            settings: Settings {
                log_level: LevelFilterSerde::try_from("DEBUG".to_string()).unwrap(),
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

type FlagGetter = fn(&'static Pointers) -> &'static dyn FlagToggler;

#[derive(Deserialize)]
#[serde(try_from = "String")]
struct FlagSpec {
    label: String,
    getter: FlagGetter,
}

impl std::fmt::Debug for FlagSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlagSpec {{ label: {:?} }}", self.label)
    }
}

impl FlagSpec {
    fn new(label: &str, getter: FlagGetter) -> FlagSpec {
        FlagSpec { label: label.to_string(), getter }
    }
}

/// Valid flag specifiers, their labels, and the pointer chains they toggle.
#[rustfmt::skip]
pub(crate) const FLAGS: &[(&str, &str, FlagGetter)] = &[
    ("one_shot", "One shot", |c| &c.one_shot),
    ("no_damage", "All no damage", |c| &c.no_damage),
    ("no_dead", "No death", |c| &c.no_dead),
    ("no_hit", "No hit", |c| &c.no_hit),
    ("no_goods_consume", "Inf Consumables", |c| &c.no_goods_consume),
    ("no_stamina_consume", "Inf Stamina", |c| &c.no_stamina_consume),
    ("no_fp_consume", "Inf Focus", |c| &c.no_fp_consume),
    ("no_ashes_of_war_fp_consume", "Inf Focus (AoW)", |c| &c.no_ashes_of_war_fp_consume),
    ("no_arrows_consume", "Inf arrows", |c| &c.no_arrows_consume),
    ("no_attack", "No attack", |c| &c.no_attack),
    ("no_move", "No move", |c| &c.no_move),
    ("no_update_ai", "No update AI", |c| &c.no_update_ai),
    ("no_trigger_event", "No trigger events", |c| &c.no_trigger_event),
    ("runearc", "Rune Arc", |c| &c.runearc),
    ("gravity", "No Gravity", |c| &c.gravity),
    ("torrent_gravity", "No Gravity (Torrent)", |c| &c.torrent_gravity),
    ("collision", "No Collision", |c| &c.collision),
    ("torrent_collision", "No Collision (Torrent)", |c| &c.torrent_collision),
    ("action_freeze", "Action freeze", |c| &c.action_freeze),
    ("display_stable_pos", "Show stable pos", |c| &c.display_stable_pos),
    ("weapon_hitbox1", "Weapon hitbox #1", |c| &c.weapon_hitbox1),
    ("weapon_hitbox2", "Weapon hitbox #2", |c| &c.weapon_hitbox2),
    ("weapon_hitbox3", "Weapon hitbox #3", |c| &c.weapon_hitbox3),
    ("hitbox_high", "High world hitbox", |c| &c.hitbox_high),
    ("hitbox_low", "Low world hitbox", |c| &c.hitbox_low),
    ("hitbox_f", "Walls hitbox", |c| &c.hitbox_f),
    ("hitbox_character", "Character hitbox", |c| &c.hitbox_character),
    ("hitbox_event", "Event hitbox", |c| &c.hitbox_event),
    ("poise_view", "Poise View", |c| &c.poise_view),
    ("sound_view", "Sound View", |c| &c.sound_view),
    ("all_targeting_view", "Targeting View", |c| &c.all_targeting_view),
    ("field_area_direction", "Direction HUD", |c| &c.field_area_direction),
    ("field_area_altimeter", "Altimeter HUD", |c| &c.field_area_altimeter),
    ("field_area_compass", "Compass HUD", |c| &c.field_area_compass),
    // ("show_map", "Show/hide map", |c| &c.show_map),
    ("show_chr", "Show/hide character", |c| &c.show_chr),
    ("show_all_map_layers", "Show all map layers", |c| &c.show_all_map_layers),
    ("show_all_graces", "Show all graces", |c| &c.show_all_graces),
];

impl TryFrom<String> for FlagSpec {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        FLAGS
            .iter()
            .find(|(id, ..)| *id == value)
            .map(|&(_, label, getter)| FlagSpec::new(label, getter))
            .ok_or_else(|| format!("\"{value}\" is not a valid flag specifier"))
    }
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
struct MultiFlagSpec {
    label: String,
    items: Vec<fn(&'static Pointers) -> &'static Bitflag<u8>>,
}

impl std::fmt::Debug for MultiFlagSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlagSpec {{ label: {:?} }}", self.label)
    }
}

impl MultiFlagSpec {
    fn new(
        label: &str,
        items: Vec<fn(&'static Pointers) -> &'static Bitflag<u8>>,
    ) -> MultiFlagSpec {
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
            e => Err(format!("\"{e}\" is not a valid multiflag specifier")),
        }
    }
}

impl ConfigSchema for Config {
    type Config = Config;

    const SETTINGS: Kind = Kind {
        name: "Settings",
        fields: &[
            ("log_level", Field::Choice(&["DEBUG", "TRACE", "INFO", "WARN", "ERROR", "OFF"])),
            ("display", Field::Key),
            ("hide", Field::OptKey),
            ("dxgi_debug", Field::Bool(false)),
            ("show_console", Field::Bool(false)),
            ("disable_update_prompt", Field::Bool(false)),
            ("radial_menu_open", Field::OptCombo),
            ("indicators", Field::Indicators),
        ],
    };
    #[rustfmt::skip]
    const WIDGETS: &'static [Kind] = &[
        Kind { name: "Flag", fields: &[("flag", Field::Flag), ("hotkey", Field::OptKey)] },
        Kind { name: "Multi flag", fields: &[("flags", Field::Flags), ("label", Field::Text("Multi flag")), ("hotkey", Field::OptKey)] },
        Kind { name: "Label", fields: &[("label", Field::Text(""))] },
        Kind { name: "Group", fields: &[("group", Field::Text("Group")), ("commands", Field::Widgets)] },
        Kind { name: "Savefile manager", fields: &[("savefile_manager", Field::KeyOrTrue)] },
        Kind { name: "Item spawner", fields: &[("item_spawner", Field::KeyOrTrue)] },
        Kind { name: "Character stats", fields: &[("character_stats", Field::KeyOrTrue)] },
        Kind { name: "Position", fields: &[("position", Field::KeyOrTrue), ("save", Field::OptKey)] },
        Kind { name: "Nudge position", fields: &[("nudge", Field::Float(1.)), ("nudge_up", Field::OptKey), ("nudge_down", Field::OptKey)] },
        Kind { name: "Cycle speed", fields: &[("cycle_speed", Field::Floats(&[0.5, 1., 2.])), ("hotkey", Field::OptKey)] },
        Kind { name: "Cycle color", fields: &[("cycle_color", Field::Ints(&[0, 1, 2, 3])), ("hotkey", Field::OptKey)] },
        Kind { name: "Runes", fields: &[("runes", Field::Int(10000)), ("hotkey", Field::OptKey)] },
        Kind { name: "Quitout", fields: &[("quitout", Field::KeyOrTrue)] },
        Kind { name: "Target", fields: &[("target", Field::KeyOrTrue)] },
        Kind { name: "Warp", fields: &[("warp", Field::Bool(true))] },
        Kind { name: "Input viewer", fields: &[("input_viewer", Field::KeyOrTrue), ("seconds", Field::Int(5))] },
    ];

    /// Also lists the specifiers that other commands accept under the `flag`
    /// key, so that the editor offers everything the parser does.
    fn flags() -> impl Iterator<Item = (&'static str, &'static str)> {
        FLAGS
            .iter()
            .map(|&(id, label, _)| (id, label))
            .chain([("show_map", "Show/hide map"), ("deathcam", "Deathcam")])
    }

    fn indicators() -> impl Iterator<Item = (&'static str, bool)> {
        INDICATORS.iter().map(|&(id, _, enabled)| (id, enabled))
    }

    fn parse(content: &str) -> Result<Config, String> {
        Config::parse(content)
    }

    fn show_cursor(show: bool) {
        POINTERS.cursor_show.set(show);
    }
}

/// Path of the configuration file, next to the DLL.
pub(crate) fn config_path() -> Option<PathBuf> {
    get_dll_path().map(|mut path| {
        path.pop();
        path.push("jdsd_er_practice_tool.toml");
        path
    })
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
