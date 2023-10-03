use libeldenring::memedit::PointerChain;

use super::Widget;
use crate::util::KeyState;

#[derive(Debug)]
pub(crate) struct ActionFreeze {
    label: String,
    ptr: PointerChain<u8>,
    hotkey: Option<KeyState>,
    state: bool,
    state_on: u8,
    state_off: u8,
}

impl ActionFreeze {
    pub(crate) fn new(ptr: PointerChain<u8>, states: (u8, u8), hotkey: Option<KeyState>) -> Self {
        let (state_off, state_on) = states;
        let state = ptr.read().map(|i| i == state_on).unwrap_or(false);
        ActionFreeze {
            label: hotkey
                .as_ref()
                .map(|hotkey| format!("Action freeze ({})", hotkey))
                .unwrap_or_else(|| "Action freeze".to_string()),
            ptr,
            state,
            state_on,
            state_off,
            hotkey,
        }
    }

    fn read(&self) -> Option<bool> {
        self.ptr.read().and_then(|val| match val {
            x if x == self.state_on => Some(true),
            x if x == self.state_off => Some(false),
            _ => None,
        })
    }

    fn set(&mut self, b: bool) {
        self.state = b;
        self.ptr.write(if self.state { self.state_on } else { self.state_off });
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

    fn interact(&mut self, ui: &imgui::Ui) {
        if ui.is_any_item_active() {
            return;
        }

        if let Some(true) = self.hotkey.as_ref().map(|k| k.keyup(ui)) {
            self.toggle();
        }
    }
}
