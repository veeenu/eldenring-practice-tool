use libeldenring::prelude::*;

use crate::util::KeyState;

use super::Widget;

#[derive(Debug)]
pub(crate) struct Runes {
    label: String,
    ptr: PointerChain<u32>,
    hotkey: KeyState,
    amount: u32,
}

impl Runes {
    pub(crate) fn new(amount: u32, ptr: PointerChain<u32>, hotkey: KeyState) -> Self {
        Runes {
            label: format!("Add {} Runes ({})", amount, hotkey),
            ptr,
            hotkey,
            amount,
        }
    }

    fn add(&self) -> Option<u32> {
        let cur_runes = self.ptr.read();

        cur_runes.map(|runes| {
            self.ptr.write(runes + self.amount);
            runes + self.amount
        })
    }
}

impl Widget for Runes {
    fn render(&mut self, ui: &imgui::Ui) {
        let scale = super::scaling_factor(ui);
        let runes = self.ptr.read();
        let _token = ui.begin_disabled(runes.is_none());

        if ui.button_with_size(&self.label, [super::BUTTON_WIDTH * scale, super::BUTTON_HEIGHT]) {
            self.add();
        }
    }

    fn interact(&mut self) {
        if self.hotkey.keyup() {
            self.add();
        }
    }
}
