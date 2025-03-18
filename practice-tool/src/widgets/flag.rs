use libeldenring::memedit::FlagToggler as FlagTogglerInner;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::flag::{Flag, FlagWidget};
use practice_tool_core::widgets::Widget;

struct FlagToggler(Box<dyn FlagTogglerInner>);

impl Flag for FlagToggler {
    fn set(&mut self, value: bool) {
        self.0.set(value);
    }

    fn get(&self) -> Option<bool> {
        self.0.get()
    }
}

pub(crate) fn flag_widget(
    label: &str,
    flag_toggler: Box<dyn FlagTogglerInner>,
    key: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(FlagWidget::new(label, FlagToggler(flag_toggler), key))
}
