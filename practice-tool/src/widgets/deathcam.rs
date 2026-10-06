use libeldenring::memedit::{Bitflag, BytesPatch, FlagToggler};
use practice_tool_core::key::Key;
use practice_tool_core::widgets::flag::{Flag, FlagWidget};
use practice_tool_core::widgets::Widget;

#[derive(Debug)]
pub(crate) struct Deathcam {
    flag: &'static Bitflag<u8>,
    flag_torrent: &'static Bitflag<u8>,
    seven: &'static BytesPatch<1>,
}

impl Deathcam {
    pub(crate) fn new(
        flag: &'static Bitflag<u8>,
        flag_torrent: &'static Bitflag<u8>,
        seven: &'static BytesPatch<1>,
    ) -> Self {
        Deathcam { flag, flag_torrent, seven }
    }
}

impl Flag for Deathcam {
    fn set(&mut self, value: bool) {
        self.seven.set(value);
        self.flag.set(value);
        self.flag_torrent.set(value);
    }

    fn get(&self) -> Option<bool> {
        self.flag.get()
    }
}

pub(crate) fn deathcam(
    flag: &'static Bitflag<u8>,
    flag_torrent: &'static Bitflag<u8>,
    seven: &'static BytesPatch<1>,
    key: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(FlagWidget::new("Deathcam", Deathcam::new(flag, flag_torrent, seven), key))
}
