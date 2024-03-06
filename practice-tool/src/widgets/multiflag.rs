use libeldenring::memedit::Bitflag;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::flag::{Flag, FlagWidget};
use practice_tool_core::widgets::Widget;

#[derive(Debug)]
struct MultiFlag {
    bitflags: Vec<Bitflag<u8>>,
}

impl MultiFlag {
    fn new(bitflags: Vec<Bitflag<u8>>) -> Self {
        Self { bitflags }
    }
}

impl Flag for MultiFlag {
    fn set(&mut self, value: bool) {
        for flag in &self.bitflags {
            flag.set(value);
        }
    }

    fn get(&self) -> Option<bool> {
        self.bitflags.first().and_then(Bitflag::get)
    }
}

pub(crate) fn multi_flag(
    label: &str,
    bitflags: Vec<Bitflag<u8>>,
    key: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(FlagWidget::new(label, MultiFlag::new(bitflags), key))
}

// impl MultiFlag {
//     pub(crate) fn new(label: &str, bitflags: Vec<Bitflag<u8>>, hotkey:
// Option<KeyState>) -> Self {         MultiFlag {
//             label: hotkey
//                 .as_ref()
//                 .map(|hotkey| format!("{} ({})", label, hotkey))
//                 .unwrap_or_else(|| label.to_string()),
//             bitflags,
//             hotkey,
//         }
//     }
//
//     fn get(&self) -> Option<bool> {
//         self.bitflags.first().and_then(Bitflag::get)
//     }
//
//     fn set(&self, state: bool) {
//         for flag in &self.bitflags {
//             flag.set(state);
//         }
//     }
//
//     fn toggle(&self) {
//         if let Some(state) = self.get() {
//             self.set(!state);
//         }
//     }
// }
//
// impl Widget for MultiFlag {
//     fn render(&mut self, ui: &imgui::Ui) {
//         let state = self.get();
//
//         if let Some(mut state) = state {
//             if ui.checkbox(&self.label, &mut state) {
//                 self.set(state);
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
//             self.toggle();
//         }
//     }
// }
