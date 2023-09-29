use std::sync::Arc;

use parking_lot::Mutex;

pub(crate) const BUTTON_WIDTH: f32 = 320.;
pub(crate) const BUTTON_HEIGHT: f32 = 0.;

pub(crate) mod action_freeze;
pub(crate) mod character_stats;
pub(crate) mod cycle_speed;
pub(crate) mod deathcam;
pub(crate) mod flag;
pub(crate) mod group;
pub(crate) mod item_spawn;
pub(crate) mod multiflag;
pub(crate) mod nudge_pos;
pub(crate) mod position;
pub(crate) mod quitout;
pub(crate) mod runes;
pub(crate) mod savefile_manager;
pub(crate) mod target;
pub(crate) mod warp;

pub(crate) trait Widget: Send + Sync + std::fmt::Debug {
    fn render(&mut self, _ui: &imgui::Ui);
    fn render_closed(&mut self, _ui: &imgui::Ui) {}
    fn interact(&mut self, _ui: &imgui::Ui) {}
    fn interact_ui(&mut self, _ui: &imgui::Ui) {}

    fn enter(&self, _ui: &imgui::Ui) -> Option<Arc<Mutex<Box<dyn Widget>>>> {
        None
    }
    fn exit(&self, _ui: &imgui::Ui) {}
    fn cursor_down(&mut self) {}
    fn cursor_up(&mut self) {}

    fn want_enter(&mut self) -> bool {
        false
    }
    fn want_exit(&mut self) -> bool {
        false
    }
    fn log(&mut self) -> Option<Vec<String>> {
        None
    }
}

pub(crate) fn scaling_factor(ui: &imgui::Ui) -> f32 {
    let width = ui.io().display_size[0];
    if width > 2000. {
        1. + 1. / 3.
    } else if width > 1200. {
        1.
    } else {
        2. / 3.
    }
}

pub(crate) fn string_match(needle: &str, haystack: &str) -> bool {
    let needle = needle.chars().flat_map(char::to_lowercase);
    let mut haystack = haystack.chars().flat_map(char::to_lowercase);

    'o: for c in needle {
        for d in &mut haystack {
            if c == d {
                continue 'o;
            }
        }
        return false;
    }
    true
}
