use std::cmp::Ordering;
use std::fmt::Write;

use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::store_value::{ReadWrite, StoreValue};
use practice_tool_core::widgets::Widget;

#[derive(Debug)]
struct CycleColor {
    ptr: PointerChain<i32>,
    values: Vec<i32>,
    current: Option<i32>,
    label: String,
}

impl CycleColor {
    fn new(values: &[i32], ptr: PointerChain<i32>) -> Self {
        let mut values = values.to_vec();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        CycleColor { ptr, values, current: None, label: String::new() }
    }
}

impl ReadWrite for CycleColor {
    fn read(&mut self) -> bool {
        self.current = self.ptr.read();

        self.label.clear();

        match self.current {
            Some(c) => write!(self.label, "Mesh Color [{c}]").ok(),
            None => write!(self.label, "Mesh Color").ok(),
        };

        self.current.is_some()
    }

    fn write(&mut self) {
        let next = *self
            .current
            .and_then(|current| self.values.iter().find(|&&x| x > current))
            .unwrap_or_else(|| self.values.first().unwrap_or(&0));

        self.ptr.write(next);
    }

    fn label(&self) -> &str {
        &self.label
    }
}

pub(crate) fn cycle_color(
    values: &[i32],
    ptr: PointerChain<i32>,
    key: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(StoreValue::new(CycleColor::new(values, ptr), key))
}
