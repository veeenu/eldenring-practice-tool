use libeldenring::memedit::FlagToggler;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::flag::{Flag, FlagWidget};
use practice_tool_core::widgets::Widget;

#[derive(Debug)]
struct MultiFlag {
    flags: Vec<Box<dyn FlagToggler>>,
}

impl MultiFlag {
    fn new(flags: Vec<Box<dyn FlagToggler>>) -> Self {
        Self { flags }
    }
}

impl Flag for MultiFlag {
    fn set(&mut self, value: bool) {
        for flag in &self.flags {
            flag.set(value);
        }
    }

    fn get(&self) -> Option<bool> {
        self.flags.first().and_then(|x| x.get())
    }
}

pub(crate) fn multi_flag(
    label: &str,
    flags: Vec<Box<dyn FlagToggler>>,
    key: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(FlagWidget::new(label, MultiFlag::new(flags), key))
}
