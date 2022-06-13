use libeldenring::prelude::*;

use crate::util::KeyState;

use super::Widget;

#[derive(Debug)]
pub(crate) struct Quitout {
    label: String,
    ptr: PointerChain<u8>,
    hotkey: KeyState,
}

impl Quitout {
    pub(crate) fn new(ptr: PointerChain<u8>, hotkey: KeyState) -> Self {
        Quitout {
            label: format!("Quitout ({})", hotkey),
            ptr,
            hotkey,
        }
    }
}

impl Widget for Quitout {
    fn render(&mut self, ui: &imgui::Ui) {
        let scale = super::scaling_factor(ui);

        if ui.button_with_size(
            &self.label,
            [super::BUTTON_WIDTH * scale, super::BUTTON_HEIGHT],
        ) {
            self.ptr.write(1);
        }
    }

    fn interact(&mut self) {
        if self.hotkey.keyup() {
            self.ptr.write(1);
        }
    }
}
