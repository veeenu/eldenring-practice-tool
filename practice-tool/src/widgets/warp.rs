use std::mem;

use hudhook::tracing::info;
use libeldenring::prelude::*;

use super::Widget;
use crate::util::KeyState;

type WarpFunc = extern "system" fn(u64, u64, u32);

#[derive(Debug)]
pub(crate) struct Warp {
    label: String,
    warp_ptr: usize,
    arg1: PointerChain<u64>,
    arg2: PointerChain<u64>,
    hotkey: KeyState,
}

impl Warp {
    pub(crate) fn new(
        warp_ptr: usize,
        arg1: PointerChain<u64>,
        arg2: PointerChain<u64>,
        hotkey: KeyState,
    ) -> Self {
        Warp { label: "Warp".to_string(), warp_ptr, arg1, arg2, hotkey }
    }

    fn warp(&mut self) {
        let warp_fn: WarpFunc = unsafe { mem::transmute(self.warp_ptr) };
        let arg1 = self.arg1.read();
        let arg2 = self.arg2.read();

        info!("{:?} {:?}", arg1, arg2);

        if let (Some(arg1), Some(arg2)) = (arg1, arg2) {
            let pos = 0x3E213247 - 0x3e8;
            warp_fn(arg1, arg2, pos);
        }
    }
}

impl Widget for Warp {
    fn render(&mut self, ui: &imgui::Ui) {
        let scale = super::scaling_factor(ui);

        if ui.button_with_size(&self.label, [super::BUTTON_WIDTH * scale, super::BUTTON_HEIGHT]) {
            self.warp();
        }
    }

    fn interact(&mut self, ui: &imgui::Ui) {
        if self.hotkey.keyup(ui) {
            self.warp();
        }
    }
}
// type PackCoordsFunc = extern "system" fn(u32, u32, u32, u32);
// type WarpFunc = extern "system" fn(u32, u32);
//
// #[derive(Debug)]
// pub(crate) struct Warp {
//     label: String,
//     pack_coords_func_ptr: usize,
//     warp_func_ptr: usize,
//     hotkey: KeyState,
// }
//
// impl Warp {
//     pub(crate) fn new(pack_coords_func_ptr: usize, warp_func_ptr: usize,
// hotkey: KeyState) -> Self {         Warp { label: format!("Warp"),
// pack_coords_func_ptr, warp_func_ptr, hotkey }     }
//
//     fn warp(&self) {
//         let pack_coords: WarpFunc = unsafe {
// mem::transmute(self.pack_coords_func_ptr) };         let warp: WarpFunc =
// unsafe { mem::transmute(self.warp_func_ptr) };     }
// }
//
// impl Widget for Warp {
//     fn render(&mut self, ui: &imgui::Ui) {
//         let scale = super::scaling_factor(ui);
//
//         if ui.button_with_size(&self.label, [super::BUTTON_WIDTH * scale,
// super::BUTTON_HEIGHT]) {             self.warp();
//         }
//     }
//
//     fn interact(&mut self, ui: &imgui::Ui) {
//         if self.hotkey.keyup(ui) {
//             self.warp();
//         }
//     }
// }
