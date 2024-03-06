use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::store_value::{ReadWrite, StoreValue};
use practice_tool_core::widgets::Widget;

struct Quitout {
    ptr: PointerChain<u8>,
}

impl Quitout {
    fn new(ptr: PointerChain<u8>) -> Self {
        Self { ptr }
    }
}

impl ReadWrite for Quitout {
    fn read(&mut self) -> bool {
        self.ptr.read().is_some()
    }

    fn write(&mut self) {
        self.ptr.write(1);
    }

    fn label(&self) -> &str {
        "Quitout"
    }
}

pub(crate) fn quitout(ptr: PointerChain<u8>, key: Option<Key>) -> Box<dyn Widget> {
    Box::new(StoreValue::new(Quitout::new(ptr), key))
}

// use super::Widget;
// use crate::util::KeyState;
//
// #[derive(Debug)]
// pub(crate) struct Quitout {
//     label: String,
//     ptr: PointerChain<u8>,
//     hotkey: KeyState,
// }
//
// impl Quitout {
//     pub(crate) fn new(ptr: PointerChain<u8>, hotkey: KeyState) -> Self {
//         Quitout { label: format!("Quitout ({})", hotkey), ptr, hotkey }
//     }
// }
//
// impl Widget for Quitout {
//     fn render(&mut self, ui: &imgui::Ui) {
//         let scale = super::scaling_factor(ui);
//
//         if ui.button_with_size(&self.label, [super::BUTTON_WIDTH * scale,
// super::BUTTON_HEIGHT]) {             self.ptr.write(1);
//         }
//     }
//
//     fn interact(&mut self, ui: &imgui::Ui) {
//         if ui.is_any_item_active() {
//             return;
//         }
//
//         if self.hotkey.keyup(ui) {
//             self.ptr.write(1);
//         }
//     }
// }
