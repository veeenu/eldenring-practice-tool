use libeldenring::memedit::PointerChain;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::flag::{Flag, FlagWidget};
use practice_tool_core::widgets::Widget;

struct ActionFreeze {
    ptr: PointerChain<u8>,
    state: bool,
    state_on: u8,
    state_off: u8,
}

impl ActionFreeze {
    fn new(ptr: PointerChain<u8>, states: (u8, u8)) -> Self {
        let (state_off, state_on) = states;
        let state = ptr.read().map(|i| i == state_on).unwrap_or(false);
        Self { ptr, state, state_on, state_off }
    }
}

impl Flag for ActionFreeze {
    fn get(&self) -> Option<bool> {
        self.ptr.read().and_then(|val| match val {
            x if x == self.state_on => Some(true),
            x if x == self.state_off => Some(false),
            _ => None,
        })
    }

    fn set(&mut self, value: bool) {
        self.state = value;
        self.ptr.write(if self.state { self.state_on } else { self.state_off });
    }
}

pub(crate) fn action_freeze(
    ptr: PointerChain<u8>,
    states: (u8, u8),
    key: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(FlagWidget::new("Action freeze", ActionFreeze::new(ptr, states), key))
}
