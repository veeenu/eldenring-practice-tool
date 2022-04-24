use crate::util::KeyState;
use libeldenring::memedit::Bitflag;

use super::Widget;

#[derive(Debug)]
pub(crate) struct MultiFlag {
    label: String,
    bitflags: Vec<Bitflag<u8>>,
    hotkey: Option<KeyState>,
}

impl MultiFlag {
    pub(crate) fn new(label: &str, bitflags: Vec<Bitflag<u8>>, hotkey: Option<KeyState>) -> Self {
        MultiFlag {
            label: hotkey
                .as_ref()
                .map(|hotkey| format!("{} ({})", label, hotkey))
                .unwrap_or_else(|| label.to_string()),
            bitflags,
            hotkey,
        }
    }

    fn get(&self) -> Option<bool> {
        self.bitflags.first().and_then(Bitflag::get)
    }

    fn set(&self, state: bool) {
        for flag in &self.bitflags {
            flag.set(state);
        }
    }

    fn toggle(&self) {
        if let Some(state) = self.get() {
            self.set(!state);
        }
    }
}

impl Widget for MultiFlag {
    fn render(&mut self, ui: &imgui::Ui) {
        let state = self.get();

        if let Some(mut state) = state {
            if ui.checkbox(&self.label, &mut state) {
                self.set(state);
            }
        } else {
            let token = ui.begin_disabled(true);
            ui.checkbox(&self.label, &mut false);
            token.end();
        }
    }

    fn interact(&mut self) {
        if let Some(true) = self.hotkey.as_ref().map(KeyState::keyup) {
            self.toggle();
        }
    }
}
