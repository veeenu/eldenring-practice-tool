use libeldenring::memedit::{Bitflag, PointerChain};
use practice_tool_core::key::Key;
use practice_tool_core::widgets::flag::{Flag, FlagWidget};
use practice_tool_core::widgets::Widget;

#[derive(Debug)]
pub(crate) struct Deathcam {
    flag: Bitflag<u8>,
    flag_torrent: Bitflag<u8>,
    seven: PointerChain<u8>,
}

impl Deathcam {
    pub(crate) fn new(
        flag: Bitflag<u8>,
        flag_torrent: Bitflag<u8>,
        seven: PointerChain<u8>,
    ) -> Self {
        Deathcam { flag, flag_torrent, seven }
    }
}

impl Flag for Deathcam {
    fn set(&mut self, value: bool) {
        if let Some(state) = self.flag.get() {
            self.seven.write(if state { 7 } else { 0 });
            self.flag.set(value);
            self.flag_torrent.set(value);
        }
    }

    fn get(&self) -> Option<bool> {
        self.flag.get()
    }
}

pub(crate) fn deathcam(
    flag: Bitflag<u8>,
    flag_torrent: Bitflag<u8>,
    seven: PointerChain<u8>,
    key: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(FlagWidget::new("Deathcam", Deathcam::new(flag, flag_torrent, seven), key))
}
