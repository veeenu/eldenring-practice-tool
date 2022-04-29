use libeldenring::memedit::PointerChain;

use crate::util::KeyState;
use super::Widget;

#[derive(Debug)]
pub(crate) struct ActionFreeze {
    label: String,
    ptr: PointerChain<u8>,
    hotkey: Option<KeyState>,
    state: bool,
}

impl ActionFreeze {
    pub(crate) fn new(ptr: PointerChain<u8>, hotkey: Option<KeyState>) -> Self {
        let state = ptr.read().map(|i| i == 0xB2).unwrap_or(false);
        ActionFreeze {
            label: hotkey
                .as_ref()
                .map(|hotkey| format!("Action freeze ({})", hotkey))
                .unwrap_or_else(|| "Action freeze".to_string()),
            ptr,
            state,
            hotkey,
        }
    }

    fn read(&self) -> Option<bool> {
        self.ptr.read().and_then(|val| match val {
            0xB2 => Some(true),
            0xB1 => Some(false),
            _ => None
        })
    }

    fn set(&mut self, b: bool) {
        self.state = b;
        self.ptr.write(if self.state { 0xB2 } else { 0xB1 });
    }

    fn toggle(&mut self) {
        self.set(!self.state);
    }
}

impl Widget for ActionFreeze {
    fn render(&mut self, ui: &imgui::Ui) {
        if let Some(mut state) = self.read() {
            if ui.checkbox(&self.label, &mut state) {
                self.set(state);
            }
        } else {
            let token = ui.begin_disabled(true);
            ui.checkbox(&self.label, &mut self.state);
            token.end();
        }
    }

    fn interact(&mut self) {
        if let Some(true) = self.hotkey.as_ref().map(KeyState::keyup) {
            self.toggle();
        }
    }
}
