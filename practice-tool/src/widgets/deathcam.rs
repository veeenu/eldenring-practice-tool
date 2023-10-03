use libeldenring::memedit::{Bitflag, PointerChain};

use super::Widget;
use crate::util::KeyState;

#[derive(Debug)]
pub(crate) struct Deathcam {
    label: String,
    flag: Bitflag<u8>,
    flag_torrent: Bitflag<u8>,
    seven: PointerChain<u8>,
    hotkey: Option<KeyState>,
}

impl Deathcam {
    pub(crate) fn new(
        flag: Bitflag<u8>,
        flag_torrent: Bitflag<u8>,
        seven: PointerChain<u8>,
        hotkey: Option<KeyState>,
    ) -> Self {
        let label = match hotkey {
            Some(k) => format!("Deathcam ({k})"),
            None => "Deathcam".to_string(),
        };
        Deathcam { label, flag, flag_torrent, seven, hotkey }
    }
}

impl Widget for Deathcam {
    fn render(&mut self, ui: &imgui::Ui) {
        let state = self.flag.get();

        if let Some(mut state) = state {
            self.seven.write(if state { 7 } else { 0 });
            if ui.checkbox(&self.label, &mut state) {
                self.flag.set(state);
                self.flag_torrent.set(state);
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
            if let Some(false) = self.flag.toggle() {
                self.seven.write(0x0);
            }
        }
    }
}
