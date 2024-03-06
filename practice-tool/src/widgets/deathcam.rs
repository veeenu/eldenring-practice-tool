use libeldenring::memedit::{Bitflag, PointerChain};
use practice_tool_core::key::Key;
use practice_tool_core::widgets::flag::{Flag, FlagWidget};
use practice_tool_core::widgets::Widget;

#[derive(Debug)]
pub(crate) struct Deathcam {
    flag: Bitflag<u8>,
    flag_torrent: Bitflag<u8>,
    seven: PointerChain<u8>,
}

impl Deathcam {
    pub(crate) fn new(
        flag: Bitflag<u8>,
        flag_torrent: Bitflag<u8>,
        seven: PointerChain<u8>,
    ) -> Self {
        Deathcam { flag, flag_torrent, seven }
    }
}

impl Flag for Deathcam {
    fn set(&mut self, value: bool) {
        if let Some(state) = self.flag.get() {
            self.seven.write(if state { 7 } else { 0 });
            self.flag.set(value);
            self.flag_torrent.set(value);
        }
    }

    fn get(&self) -> Option<bool> {
        self.flag.get()
    }
}

pub(crate) fn deathcam(
    flag: Bitflag<u8>,
    flag_torrent: Bitflag<u8>,
    seven: PointerChain<u8>,
    key: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(FlagWidget::new("Deathcam", Deathcam::new(flag, flag_torrent, seven), key))
}

// impl Widget for Deathcam {
//     fn render(&mut self, ui: &imgui::Ui) {
//         let state = self.flag.get();
//
//         if let Some(mut state) = state {
//             self.seven.write(if state { 7 } else { 0 });
//             if ui.checkbox(&self.label, &mut state) {
//                 self.flag.set(state);
//                 self.flag_torrent.set(state);
//             }
//         } else {
//             let token = ui.begin_disabled(true);
//             ui.checkbox(&self.label, &mut false);
//             token.end();
//         }
//     }
//
//     fn interact(&mut self, ui: &imgui::Ui) {
//         if ui.is_any_item_active() {
//             return;
//         }
//
//         if let Some(true) = self.hotkey.as_ref().map(|k| k.keyup(ui)) {
//             if let Some(false) = self.flag.toggle() {
//                 self.seven.write(0x0);
//             }
//         }
//     }
// }
