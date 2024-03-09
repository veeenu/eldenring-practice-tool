use std::cmp::Ordering;
use std::fmt::Write;

use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::store_value::{ReadWrite, StoreValue};
use practice_tool_core::widgets::Widget;

#[derive(Debug)]
struct CycleSpeed {
    ptr: [PointerChain<f32>; 2],
    values: Vec<f32>,
    current: Option<f32>,
    label: String,
}

impl CycleSpeed {
    fn new(values: &[f32], ptr: [PointerChain<f32>; 2]) -> Self {
        let mut values = values.to_vec();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        CycleSpeed { ptr, values, current: None, label: String::new() }
    }
}

impl ReadWrite for CycleSpeed {
    fn read(&mut self) -> bool {
        self.current = self.ptr[0].read();

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

        self.ptr[0].write(next);
        self.ptr[1].write(next);
    }

    fn label(&self) -> &str {
        &self.label
    }
}

pub(crate) fn cycle_speed(
    values: &[f32],
    ptr: [PointerChain<f32>; 2],
    key: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(StoreValue::new(CycleSpeed::new(values, ptr), key))
}
