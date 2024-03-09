use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::store_value::{ReadWrite, StoreValue};
use practice_tool_core::widgets::Widget;

struct Runes {
    ptr: PointerChain<u32>,
    current: u32,
    amount: u32,
    label: String,
}

impl Runes {
    fn new(amount: u32, ptr: PointerChain<u32>) -> Self {
        Self { ptr, current: 0, amount, label: format!("Add {amount} runes") }
    }
}

impl ReadWrite for Runes {
    fn read(&mut self) -> bool {
        if let Some(current) = self.ptr.read() {
            self.current = current;
            true
        } else {
            false
        }
    }

    fn write(&mut self) {
        self.ptr.write(self.current + self.amount);
    }

    fn label(&self) -> &str {
        &self.label
    }
}

pub(crate) fn runes(amount: u32, ptr: PointerChain<u32>, key: Option<Key>) -> Box<dyn Widget> {
    Box::new(StoreValue::new(Runes::new(amount, ptr), key))
}
