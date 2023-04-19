use hudhook::tracing::debug;
use imgui::*;
use libeldenring::prelude::*;

use super::Widget;
use crate::util::KeyState;

#[derive(Debug)]
pub(crate) struct CharacterStatsEdit {
    hotkey_open: KeyState,
    hotkey_close: KeyState,
    label_open: String,
    label_close: String,
    ptr: PointerChain<CharacterStats>,
    stats: Option<CharacterStats>,
}

impl CharacterStatsEdit {
    pub(crate) fn new(
        hotkey_open: KeyState,
        hotkey_close: KeyState,
        ptr: PointerChain<CharacterStats>,
    ) -> Self {
        let label_open = format!("Edit stats ({})", hotkey_open);
        let label_close = format!("Close ({})", hotkey_close);
        CharacterStatsEdit { hotkey_open, hotkey_close, label_open, label_close, ptr, stats: None }
    }
}

impl Widget for CharacterStatsEdit {
    fn render(&mut self, ui: &imgui::Ui) {
        let button_width = super::BUTTON_WIDTH * super::scaling_factor(ui);
        if ui.button_with_size(&self.label_open, [button_width, super::BUTTON_HEIGHT]) {
            self.stats = self.ptr.read();
            debug!("{:?}", self.stats);
        }

        if self.stats.is_some() {
            ui.open_popup("##character_stats_edit");
        }

        let style_tokens =
            [ui.push_style_color(imgui::StyleColor::ModalWindowDimBg, super::MODAL_BACKGROUND)];

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

            if self.hotkey_close.keyup(ui)
                || ui.button_with_size(&self.label_close, [button_width, super::BUTTON_HEIGHT])
            {
                ui.close_current_popup();
                self.stats.take();
            }
        }

        style_tokens.into_iter().rev().for_each(|t| t.pop());
    }

    fn interact(&mut self, ui: &imgui::Ui) {
        if self.hotkey_open.keyup(ui) {
            self.stats = self.ptr.read();
        }
    }
}
