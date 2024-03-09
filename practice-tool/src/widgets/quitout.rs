use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::store_value::{ReadWrite, StoreValue};
use practice_tool_core::widgets::Widget;

struct Quitout {
    ptr: PointerChain<u8>,
}

impl Quitout {
    fn new(ptr: PointerChain<u8>) -> Self {
        Self { ptr }
    }
}

impl ReadWrite for Quitout {
    fn read(&mut self) -> bool {
        self.ptr.read().is_some()
    }

    fn write(&mut self) {
        self.ptr.write(1);
    }

    fn label(&self) -> &str {
        "Quitout"
    }
}

pub(crate) fn quitout(ptr: PointerChain<u8>, key: Option<Key>) -> Box<dyn Widget> {
    Box::new(StoreValue::new(Quitout::new(ptr), key))
}
