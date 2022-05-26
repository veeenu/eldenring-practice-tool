use std::cmp::Ordering;

use crate::util::KeyState;
use libeldenring::prelude::*;

use super::Widget;

#[derive(Debug)]
pub(crate) struct CycleSpeed {
    ptr: [PointerChain<f32>; 2],
    hotkey: KeyState,
    values: Vec<f32>,
}

impl CycleSpeed {
    pub(crate) fn new(values: &[f32], ptr: [PointerChain<f32>; 2], hotkey: KeyState) -> Self {
        let mut values = values.to_vec();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        CycleSpeed {
            ptr,
            hotkey,
            values,
        }
    }

    fn cycle(&self) -> Option<f32> {
        let next = self.ptr[0].read().map(|speed| {
            *self
                .values
                .iter()
                .find(|&&x| x > speed)
                .unwrap_or_else(|| self.values.get(0).unwrap_or(&1.0))
        });
        if let Some(speed) = next {
            self.ptr[0].write(speed);
            self.ptr[1].write(speed);
        }
        next
    }
}

impl Widget for CycleSpeed {
    fn render(&mut self, ui: &imgui::Ui) {
        let speed = self.ptr[0].read();
        let _token = ui.begin_disabled(speed.is_none());

        let label = if let Some(speed) = speed {
            format!("Speed [{:.1}x] ({})", speed, self.hotkey)
        } else {
            format!("Speed ({})", self.hotkey)
        };

        if ui.button_with_size(
            &label,
            [
                super::BUTTON_WIDTH * super::scaling_factor(ui),
                super::BUTTON_HEIGHT,
            ],
        ) {
            self.cycle();
        }
    }

    fn interact(&mut self) {
        if self.hotkey.keyup() {
            self.cycle();
        }
    }
}
