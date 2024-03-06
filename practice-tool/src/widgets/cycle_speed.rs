use std::cmp::Ordering;
use std::fmt::Write;

use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::store_value::{ReadWrite, StoreValue};
use practice_tool_core::widgets::Widget;

#[derive(Debug)]
struct CycleSpeed {
    ptr: PointerChain<f32>,
    values: Vec<f32>,
    current: Option<f32>,
    label: String,
}

impl CycleSpeed {
    fn new(values: &[f32], ptr: PointerChain<f32>) -> Self {
        let mut values = values.to_vec();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        CycleSpeed { ptr, values, current: None, label: String::new() }
    }
}

impl ReadWrite for CycleSpeed {
    fn read(&mut self) -> bool {
        self.current = self.ptr.read();

        self.label.clear();

        match self.current {
            Some(c) => write!(self.label, "Speed [{:.1}x]", c).ok(),
            None => write!(self.label, "Speed").ok(),
        };

        self.current.is_some()
    }

    fn write(&mut self) {
        let next = *self
            .current
            .and_then(|current| self.values.iter().find(|&&x| x > current))
            .unwrap_or_else(|| self.values.first().unwrap_or(&1.0));

        self.ptr.write(next);
    }

    fn label(&self) -> &str {
        &self.label
    }
}

pub(crate) fn cycle_speed(values: &[f32], ptr: PointerChain<f32>, key: Key) -> Box<dyn Widget> {
    Box::new(StoreValue::new(CycleSpeed::new(values, ptr), Some(key)))
}

// use super::Widget;
// use crate::util::KeyState;
//
// #[derive(Debug)]
// pub(crate) struct CycleSpeed {
//     ptr: [PointerChain<f32>; 2],
//     hotkey: KeyState,
//     values: Vec<f32>,
// }
//
// impl CycleSpeed {
//     pub(crate) fn new(values: &[f32], ptr: [PointerChain<f32>; 2], hotkey:
// KeyState) -> Self {         let mut values = values.to_vec();
//         values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
//         CycleSpeed { ptr, hotkey, values }
//     }
//
//     fn cycle(&self) -> Option<f32> {
//         let next = self.ptr[0].read().map(|speed| {
//             *self
//                 .values
//                 .iter()
//                 .find(|&&x| x > speed)
//                 .unwrap_or_else(|| self.values.first().unwrap_or(&1.0))
//         });
//         if let Some(speed) = next {
//             self.ptr[0].write(speed);
//             self.ptr[1].write(speed);
//         }
//         next
//     }
// }
//
// impl Widget for CycleSpeed {
//     fn render(&mut self, ui: &imgui::Ui) {
//         let speed = self.ptr[0].read();
//         let _token = ui.begin_disabled(speed.is_none());
//
//         let label = if let Some(speed) = speed {
//             format!("Speed [{:.1}x] ({})", speed, self.hotkey)
//         } else {
//             format!("Speed ({})", self.hotkey)
//         };
//
//         if ui.button_with_size(label, [
//             super::BUTTON_WIDTH * super::scaling_factor(ui),
//             super::BUTTON_HEIGHT,
//         ]) {
//             self.cycle();
//         }
//     }
//
//     fn interact(&mut self, ui: &imgui::Ui) {
//         if ui.is_any_item_active() {
//             return;
//         }
//
//         if self.hotkey.keyup(ui) {
//             self.cycle();
//         }
//     }
// }
