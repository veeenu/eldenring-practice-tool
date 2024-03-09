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
