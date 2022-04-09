use std::cmp::Ordering;

use libeldenring::prelude::*;
use crate::util::KeyState;

use super::Widget;

#[derive(Debug)]
pub(crate) struct CycleSpeed {
    label: String,
    ptr: PointerChain<f32>,
    hotkey: KeyState,
    values: Vec<f32>,
}

impl CycleSpeed {
    pub(crate) fn new(values: &[f32], ptr: PointerChain<f32>, hotkey: KeyState) -> Self {
        let mut values = values.to_vec();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        CycleSpeed {
            label: format!("Speed ({})", hotkey),
            ptr,
            hotkey,
            values,
        }
    }

    fn cycle(&self) -> Option<f32> {
        let next = self.ptr.read().map(|speed| {
            *self
                .values
                .iter()
                .find(|&&x| x > speed)
                .unwrap_or_else(|| self.values.get(0).unwrap_or(&1.0))
        });
        next.map(|speed| self.ptr.write(speed));
        next
    }
}

impl Widget for CycleSpeed {
    fn render(&mut self, ui: &imgui::Ui) {
        let speed = self.ptr.read();
        let _token = ui.begin_disabled(speed.is_none());

        if ui.button_with_size(&self.label, [super::BUTTON_WIDTH, super::BUTTON_HEIGHT]) {
            self.cycle();
        }
        ui.same_line();

        if let Some(speed) = speed {
            ui.text(format!("[{:10.2}]", speed));
        } else {
            ui.text("[          ]");
        }
    }

    fn interact(&mut self) {
        if self.hotkey.keyup() {
            self.cycle();
        }
    }
}
