use hudhook::tracing::debug;
use imgui::sys::{igGetCursorPosX, igGetCursorPosY, igGetWindowPos, igSetNextWindowPos, ImVec2};
use imgui::*;
use libeldenring::prelude::*;

use super::{scaling_factor, Widget, BUTTON_HEIGHT, BUTTON_WIDTH};
use crate::util::KeyState;

#[derive(Debug)]
pub(crate) struct CharacterStatsEdit {
    ptr: PointerChain<CharacterStats>,
    stats: Option<CharacterStats>,
    hotkey_close: KeyState,
    label_close: String,
}

impl CharacterStatsEdit {
    pub(crate) fn new(ptr: PointerChain<CharacterStats>, hotkey_close: KeyState) -> Self {
        let label_close = format!("Close ({hotkey_close})");
        CharacterStatsEdit { ptr, stats: None, hotkey_close, label_close }
    }
}

impl Widget for CharacterStatsEdit {
    fn render(&mut self, ui: &imgui::Ui) {
        let scale = scaling_factor(ui);
        let button_width = BUTTON_WIDTH * scale;

        let (x, y) = unsafe {
            let mut wnd_pos = ImVec2::default();
            igGetWindowPos(&mut wnd_pos);
            (igGetCursorPosX() + wnd_pos.x, igGetCursorPosY() + wnd_pos.y)
        };

        if ui.button_with_size("Edit stats", [button_width, BUTTON_HEIGHT]) {
            self.stats = self.ptr.read();
            debug!("{:?}", self.stats);
        }

        if self.stats.is_some() {
            ui.open_popup("##character_stats_edit");
        }

        unsafe {
            igSetNextWindowPos(
                ImVec2::new(x + 200. * scale, y),
                Condition::Always as i8 as _,
                ImVec2::new(0., 0.),
            )
        };

        if let Some(_token) = ui
            .modal_popup_config("##character_stats_edit")
            .flags(
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR,
            )
            .begin_popup()
        {
            let _tok = ui.push_item_width(150.);
            if let Some(stats) = self.stats.as_mut() {
                if ui.input_int("Level", &mut stats.level).build() {
                    stats.level = stats.level.clamp(1, i32::MAX);
                }
                if ui.input_int("Vigor", &mut stats.vigor).build() {
                    stats.vigor = stats.vigor.clamp(1, 99);
                }
                if ui.input_int("Mind", &mut stats.mind).build() {
                    stats.mind = stats.mind.clamp(1, 99);
                }
                if ui.input_int("Endurance", &mut stats.endurance).build() {
                    stats.endurance = stats.endurance.clamp(1, 99);
                }
                if ui.input_int("Strength", &mut stats.strength).build() {
                    stats.strength = stats.strength.clamp(1, 99);
                }
                if ui.input_int("Dexterity", &mut stats.dexterity).build() {
                    stats.dexterity = stats.dexterity.clamp(1, 99);
                }
                if ui.input_int("Intelligence", &mut stats.intelligence).build() {
                    stats.intelligence = stats.intelligence.clamp(1, 99);
                }
                if ui.input_int("Faith", &mut stats.faith).build() {
                    stats.faith = stats.faith.clamp(1, 99);
                }
                if ui.input_int("Arcane", &mut stats.arcane).build() {
                    stats.arcane = stats.arcane.clamp(1, 99);
                }
                if ui.input_int("Runes", &mut stats.runes).build() {
                    stats.runes = stats.runes.clamp(1, i32::MAX);
                }

                if ui.button_with_size("Apply", [button_width, super::BUTTON_HEIGHT]) {
                    self.ptr.write(stats.clone());
                }
            }

            if ui.button_with_size(&self.label_close, [button_width, super::BUTTON_HEIGHT])
                || (self.hotkey_close.keyup(ui) && !ui.is_any_item_active())
            {
                ui.close_current_popup();
                self.stats.take();
            }
        }
    }
}
