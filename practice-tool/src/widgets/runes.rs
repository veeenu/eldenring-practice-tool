use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::store_value::{ReadWrite, StoreValue};
use practice_tool_core::widgets::Widget;

struct Runes {
    ptr: PointerChain<u32>,
    current: u32,
    amount: u32,
    label: String,
}

impl Runes {
    fn new(amount: u32, ptr: PointerChain<u32>) -> Self {
        Self { ptr, current: 0, amount, label: format!("Add {amount} runes") }
    }
}

impl ReadWrite for Runes {
    fn read(&mut self) -> bool {
        if let Some(current) = self.ptr.read() {
            self.current = current;
            true
        } else {
            false
        }
    }

    fn write(&mut self) {
        self.ptr.write(self.current + self.amount);
    }

    fn label(&self) -> &str {
        &self.label
    }
}

pub(crate) fn runes(amount: u32, ptr: PointerChain<u32>, key: Option<Key>) -> Box<dyn Widget> {
    Box::new(StoreValue::new(Runes::new(amount, ptr), key))
}

// use super::Widget;
// use crate::util::KeyState;
//
// #[derive(Debug)]
// pub(crate) struct Runes {
//     label: String,
//     ptr: PointerChain<u32>,
//     hotkey: KeyState,
//     amount: u32,
// }
//
// impl Runes {
//     pub(crate) fn new(amount: u32, ptr: PointerChain<u32>, hotkey: KeyState)
// -> Self {         Runes { label: format!("Add {} Runes ({})", amount,
// hotkey), ptr, hotkey, amount }     }
//
//     fn add(&self) -> Option<u32> {
//         let cur_runes = self.ptr.read();
//
//         cur_runes.map(|runes| {
//             self.ptr.write(runes + self.amount);
//             runes + self.amount
//         })
//     }
// }
//
// impl Widget for Runes {
//     fn render(&mut self, ui: &imgui::Ui) {
//         let scale = super::scaling_factor(ui);
//         let runes = self.ptr.read();
//         let _token = ui.begin_disabled(runes.is_none());
//
//         if ui.button_with_size(&self.label, [super::BUTTON_WIDTH * scale,
// super::BUTTON_HEIGHT]) {             self.add();
//         }
//     }
//
//     fn interact(&mut self, ui: &imgui::Ui) {
//         if ui.is_any_item_active() {
//             return;
//         }
//
//         if self.hotkey.keyup(ui) {
//             self.add();
//         }
//     }
// }
