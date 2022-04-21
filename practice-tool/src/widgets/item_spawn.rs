use super::Widget;
use crate::util::KeyState;
use libeldenring::prelude::*;

use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::Display;
use std::lazy::SyncLazy;

use imgui::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ItemIDMaster(HashMap<String, ItemIDGroup>);

#[derive(Debug, Deserialize)]
struct ItemIDGroup(HashMap<String, u32>);

const ISP_TAG: &str = "##item-spawn";
// static ITEM_ID_TREE: SyncLazy<ItemIDMaster> =
//     SyncLazy::new(|| serde_json::from_str(include_str!("item_ids.json")).unwrap());
use super::item_ids::ITEM_ID_TREE;

#[derive(Debug)]
pub(crate) struct ItemSpawner {
    func_ptr: usize,
    map_item_man: usize,
    hotkey: KeyState,
    qty: u32,
    item_id: u32,
}

impl ItemSpawner {
    pub(crate) fn new(func_ptr: usize, map_item_man: usize, hotkey: KeyState) -> Self {
        ItemSpawner {
            func_ptr,
            map_item_man,
            hotkey,
            qty: 1,
            item_id: 0x40000000 + 2919,
        }
    }

    fn spawn(&self) {
        let i = ItemSpawnInstance {
            spawn_item_func_ptr: self.func_ptr as _,
            map_item_man: self.map_item_man as _,
            qty: self.qty,
            item_id: self.item_id,
        };
        unsafe {
            i.spawn();
        }
    }
}

impl Widget for ItemSpawner {
    fn render(&mut self, ui: &imgui::Ui) {
        if ui.button_with_size("Spawn item", [super::BUTTON_WIDTH, super::BUTTON_HEIGHT]) {
            ui.open_popup(ISP_TAG);
        }
        let [cx, cy] = ui.cursor_pos();
        let [wx, wy] = ui.window_pos();
        let [x, y] = [cx + wx, cy + wy - super::BUTTON_HEIGHT];
        unsafe {
            imgui_sys::igSetNextWindowPos(
                imgui_sys::ImVec2 { x, y },
                Condition::Always as _,
                imgui_sys::ImVec2 { x: 0., y: 0. },
            )
        };

        let style_tokens =
            [ui.push_style_color(imgui::StyleColor::ModalWindowDimBg, [0., 0., 0., 0.])];

        if let Some(_token) = PopupModal::new(ISP_TAG)
            .flags(
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::ALWAYS_AUTO_RESIZE,
            )
            .begin_popup(ui)
        {
            ChildWindow::new("##item-spawn-list")
                .size([240., 240.])
                .build(ui, || {
                    for (label, items) in &*ITEM_ID_TREE {
                        TreeNode::<&str>::new(*label)
                            .label::<&str, &str>(label)
                            .flags(TreeNodeFlags::SPAN_AVAIL_WIDTH)
                            .build(ui, || {
                                for (label, item_id) in items {
                                    TreeNode::<&str>::new("")
                                        .label::<&str, &str>(&label)
                                        .flags(if item_id == &self.item_id {
                                            TreeNodeFlags::LEAF | TreeNodeFlags::SELECTED
                                        } else {
                                            TreeNodeFlags::LEAF
                                        })
                                        .build(ui, || {});
                                    if ui.is_item_clicked() {
                                        self.item_id = *item_id;
                                    }
                                }
                            });
                    }
                });

            Slider::new("Qty", 0, 256).build(ui, &mut self.qty);
            if ui.button_with_size(format!("Spawn item ({})", self.hotkey), [240., 20.]) {
                self.spawn();
            }

            if ui.button_with_size("Close", [240., 20.]) {
                ui.close_current_popup();
            }
        }

        style_tokens.into_iter().rev().for_each(|t| t.pop());
    }

    // fn interact(&mut self) {}
}

#[derive(Debug)]
struct ItemSpawnInstance {
    spawn_item_func_ptr: u64,
    map_item_man: u64,
    qty: u32,
    item_id: u32,
}

impl Display for ItemSpawnInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x} (qty={})", self.item_id, self.qty,)
    }
}

impl ItemSpawnInstance {
    unsafe fn spawn(&self) {
        #[repr(C)]
        struct SpawnRequest {
            item_id: u32,
            qty: u32,
            unk1: u32,
            unk2: u32,
        }

        type SpawnItemFn = extern "system" fn(*const c_void, *mut SpawnRequest, *mut [u32; 6]);
        let spawn_fn_ptr = std::mem::transmute::<_, SpawnItemFn>(self.spawn_item_func_ptr);
        let pp_map_item_man = self.map_item_man as *const *const c_void;

        let item_id = self.item_id;
        let qty = self.qty;

        let mut spawn_request = SpawnRequest {
            item_id,
            qty,
            unk1: item_id & (0x10000000 | 0x20000000 | 0x40000000 | 0x80000000),
            unk2: 0xffffffff,
        };

        spawn_fn_ptr(
            *pp_map_item_man,
            &mut spawn_request as *mut _,
            &mut [0u32; 6] as *mut _,
        );
    }
}

// 0x00000000 -- Weapon
// 0x10000000 -- Protector
// 0x20000000 -- Accessory
// 0x40000000 -- Goods
// 0x80000000 -- Gem
