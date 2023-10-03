use libeldenring::memedit::Bitflag;

use super::Widget;
use crate::util::KeyState;

#[derive(Debug)]
pub(crate) struct Flag {
    label: String,
    bitflag: Bitflag<u8>,
    hotkey: Option<KeyState>,
}

impl Flag {
    pub(crate) fn new(label: &str, bitflag: Bitflag<u8>, hotkey: Option<KeyState>) -> Self {
        Flag {
            label: hotkey
                .as_ref()
                .map(|hotkey| format!("{} ({})", label, hotkey))
                .unwrap_or_else(|| label.to_string()),
            bitflag,
            hotkey,
        }
    }
}

impl Widget for Flag {
    fn render(&mut self, ui: &imgui::Ui) {
        let state = self.bitflag.get();

        if let Some(mut state) = state {
            if ui.checkbox(&self.label, &mut state) {
                self.bitflag.set(state);
            }
        } else {
            let token = ui.begin_disabled(true);
            ui.checkbox(&self.label, &mut false);
            token.end();
        }
    }

    fn interact(&mut self, ui: &imgui::Ui) {
        if ui.is_any_item_active() {
            return;
        }

        if let Some(true) = self.hotkey.as_ref().map(|k| k.keyup(ui)) {
            self.bitflag.toggle();
        }
    }
}
