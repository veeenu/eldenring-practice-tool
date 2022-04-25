use libeldenring::prelude::*;

use super::Widget;
use crate::util::KeyState;

use std::borrow::Cow;
use std::ffi::c_void;
use std::fmt::Display;
use std::lazy::SyncLazy;

use imgui::*;
use serde::Deserialize;

static AFFINITIES: [(u32, &str); 13] = [
    (0, "No affinity"),
    (100, "Heavy"),
    (200, "Keen"),
    (300, "Quality"),
    (400, "Fire"),
    (500, "Flame Art"),
    (600, "Lightning"),
    (700, "Sacred"),
    (800, "Magic"),
    (900, "Cold"),
    (1000, "Poison"),
    (1100, "Blood"),
    (1200, "Occult"),
];

static UPGRADES: [(u32, &str); 26] = [
    (0, "+0"),
    (1, "+1"),
    (2, "+2"),
    (3, "+3"),
    (4, "+4"),
    (5, "+5"),
    (6, "+6"),
    (7, "+7"),
    (8, "+8"),
    (9, "+9"),
    (10, "+10"),
    (11, "+11"),
    (12, "+12"),
    (13, "+13"),
    (14, "+14"),
    (15, "+15"),
    (16, "+16"),
    (17, "+17"),
    (18, "+18"),
    (19, "+19"),
    (20, "+20"),
    (21, "+21"),
    (22, "+22"),
    (23, "+23"),
    (24, "+24"),
    (25, "+25"),
];

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ItemIDNode {
    Leaf {
        node: String,
        value: u32,
    },
    Node {
        node: String,
        children: Vec<ItemIDNode>,
    },
}

#[derive(Debug)]
enum ItemIDNodeRef<'a> {
    Leaf {
        node: &'a str,
        value: u32,
    },
    Node {
        node: &'a str,
        children: Vec<ItemIDNodeRef<'a>>,
    },
}

impl<'a> ItemIDNodeRef<'a> {
    fn render(&self, ui: &imgui::Ui, current: &mut u32, filtered: bool) {
        match self {
            ItemIDNodeRef::Leaf { node, value } => {
                unsafe { imgui_sys::igUnindent(imgui_sys::igGetTreeNodeToLabelSpacing()) };
                TreeNode::<&str>::new(*node)
                    .label::<&str, &str>(node)
                    .flags(if current == value {
                        TreeNodeFlags::LEAF
                            | TreeNodeFlags::SELECTED
                            | TreeNodeFlags::NO_TREE_PUSH_ON_OPEN
                    } else {
                        TreeNodeFlags::LEAF | TreeNodeFlags::NO_TREE_PUSH_ON_OPEN
                    })
                    .build(ui, || {});
                unsafe { imgui_sys::igIndent(imgui_sys::igGetTreeNodeToLabelSpacing()) };
                if ui.is_item_clicked() {
                    *current = *value;
                }
            }
            ItemIDNodeRef::Node { node, children } => {
                let n = TreeNode::<&str>::new(*node).label::<&str, &str>(node);

                let n = if filtered {
                    n.opened(filtered, Condition::Always)
                } else {
                    n
                };

                n.flags(TreeNodeFlags::SPAN_AVAIL_WIDTH).build(ui, || {
                    for node in children {
                        node.render(ui, current, filtered);
                    }
                });
            }
        }
    }
}

impl<'a> From<&'a ItemIDNode> for ItemIDNodeRef<'a> {
    fn from(v: &'a ItemIDNode) -> Self {
        match v {
            ItemIDNode::Leaf { node, value } => ItemIDNodeRef::Leaf {
                node,
                value: *value,
            },
            ItemIDNode::Node { node, children } => ItemIDNodeRef::Node {
                node,
                children: children.iter().map(ItemIDNodeRef::from).collect(),
            },
        }
    }
}

impl ItemIDNode {
    fn filter(&self, filter: &str) -> Option<ItemIDNodeRef> {
        if filter.is_empty() {
            Some(ItemIDNodeRef::from(self))
        } else {
            match self {
                ItemIDNode::Leaf { node, value } => {
                    if string_match(filter, node) {
                        Some(ItemIDNodeRef::Leaf {
                            node,
                            value: *value,
                        })
                    } else {
                        None
                    }
                }
                ItemIDNode::Node { node, children } => {
                    let children: Vec<_> = children
                        .iter()
                        .filter_map(|c| c.filter(filter).map(ItemIDNodeRef::from))
                        .collect();
                    if children.is_empty() {
                        None
                    } else {
                        Some(ItemIDNodeRef::Node {
                            node,
                            children,
                        })
                    }
                }
            }
        }
    }
}

fn string_match(needle: &str, haystack: &str) -> bool {
    let needle = needle.chars().flat_map(char::to_lowercase);
    let mut haystack = haystack.chars().flat_map(char::to_lowercase);

    'o: for c in needle {
        for d in &mut haystack {
            if c == d {
                continue 'o;
            }
        }
        return false;
    }
    true
}

const ISP_TAG: &str = "##item-spawn";
static ITEM_ID_TREE: SyncLazy<Vec<ItemIDNode>> =
    SyncLazy::new(|| serde_json::from_str(include_str!("item_ids.json")).unwrap());

#[derive(Debug)]
pub(crate) struct ItemSpawner<'a> {
    func_ptr: usize,
    map_item_man: usize,
    hotkey_load: KeyState,
    hotkey_close: KeyState,
    sentinel: Bitflag<u8>,

    label_load: String,
    label_close: String,

    qty: u32,
    item_id: u32,
    upgrade: usize,
    affinity: usize,

    filter_string: String,
    log: Option<Vec<String>>,
    item_id_tree: Vec<ItemIDNodeRef<'a>>,
}

impl ItemSpawner<'_> {
    pub(crate) fn new(
        func_ptr: usize,
        map_item_man: usize,
        sentinel: Bitflag<u8>,
        hotkey_load: KeyState,
        hotkey_close: KeyState,
    ) -> Self {
        let label_load = format!("Spawn item ({})", hotkey_load);
        let label_close = format!("Close ({})", hotkey_close);
        ItemSpawner {
            func_ptr,
            map_item_man,
            hotkey_load,
            hotkey_close,
            label_load,
            label_close,
            sentinel,
            qty: 1,
            item_id: 0x40000000 + 2919,
            upgrade: 0,
            affinity: 0,
            filter_string: String::new(),
            log: None,
            item_id_tree: ITEM_ID_TREE.iter().map(ItemIDNodeRef::from).collect(),
        }
    }

    fn spawn(&mut self) {
        if self.sentinel.get().is_none() {
            self.write_log("Not spawning item when not in game".into());
            return;
        }

        let upgrade = UPGRADES[self.upgrade].0;
        let affinity = AFFINITIES[self.affinity].0;

        let i = ItemSpawnInstance {
            spawn_item_func_ptr: self.func_ptr as _,
            map_item_man: self.map_item_man as _,
            qty: self.qty,
            item_id: self.item_id + upgrade + affinity,
        };

        self.write_log(format!(
            "Spawning {} #{} {} {}",
            i.qty, self.item_id, UPGRADES[self.upgrade].1, AFFINITIES[self.affinity].1,
        ));

        unsafe {
            i.spawn();
        }
    }

    fn write_log(&mut self, log: String) {
        let logs = self.log.take();
        self.log = match logs {
            Some(mut v) => {
                v.push(log);
                Some(v)
            }
            None => Some(vec![log]),
        };
    }
}

impl Widget for ItemSpawner<'_> {
    fn render(&mut self, ui: &imgui::Ui) {
        if ui.button_with_size("Spawn item", [super::BUTTON_WIDTH, super::BUTTON_HEIGHT]) {
            ui.open_popup(ISP_TAG);
        }

        let style_tokens =
            [ui.push_style_color(imgui::StyleColor::ModalWindowDimBg, [0., 0., 0., 0.])];

        if let Some(_token) = PopupModal::new(ISP_TAG)
            .flags(
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR,
            )
            .begin_popup(ui)
        {
            {
                let _tok = ui.push_item_width(-1.);
                if InputText::new(ui, "##item-spawn-filter", &mut self.filter_string)
                    .hint("Filter...")
                    .build()
                {
                    self.item_id_tree = ITEM_ID_TREE
                        .iter()
                        .filter_map(|n| n.filter(&self.filter_string))
                        .collect();
                }
            }
            ChildWindow::new("##item-spawn-list")
                .size([240., 200.])
                .build(ui, || {
                    for node in &self.item_id_tree {
                        node.render(ui, &mut self.item_id, !self.filter_string.is_empty());
                    }
                });

            ui.set_next_item_width(110.);
            ui.combo(
                "##item-spawn-affinity",
                &mut self.affinity,
                &AFFINITIES,
                |(_, label)| Cow::Borrowed(label),
            );

            // TODO figure out why upgrades don't work.
            ui.same_line();
            ui.set_next_item_width(110.);
            ui.combo(
                "##item-spawn-upgrade",
                &mut self.upgrade,
                &UPGRADES,
                |(_, label)| Cow::Borrowed(label),
            );

            Slider::new("Qty", 1, 99).build(ui, &mut self.qty);
            if self.hotkey_load.keyup() || ui.button_with_size(&self.label_load, [240., 20.]) {
                self.spawn();
            }

            if self.hotkey_close.keyup() || ui.button_with_size(&self.label_close, [240., 20.]) {
                ui.close_current_popup();
            }
        }

        style_tokens.into_iter().rev().for_each(|t| t.pop());
    }

    fn log(&mut self) -> Option<Vec<String>> {
        self.log.take()
    }

    fn interact(&mut self) {
        if self.hotkey_load.keyup() {
            self.spawn();
        }
    }
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
        // 550300 in 102
        #[repr(C)]
        struct SpawnRequest {
            one: u32,
            item_id: u32,
            qty: u32,
        }

        // let module_base_addr = windows::Win32::System::LibraryLoader::GetModuleHandleA(windows::core::PCSTR(std::ptr::null_mut())).0 as usize;

        type SpawnItemFn = extern "system" fn(*const c_void, *mut SpawnRequest, *mut u32, u32);
        // let spawn_fn_ptr = std::mem::transmute::<_, SpawnItemFn>(module_base_addr + 0x54E570);
        let spawn_fn_ptr = std::mem::transmute::<_, SpawnItemFn>(self.spawn_item_func_ptr);
        let pp_map_item_man = self.map_item_man as *const *const c_void;

        let item_id = self.item_id;
        let qty = self.qty;
        let mut spawn_request = SpawnRequest {
            one: 1,
            item_id,
            qty,
        };
        let mut dur: u32 = 0xffffffff;

        spawn_fn_ptr(
            *pp_map_item_man,
            &mut spawn_request as *mut _,
            &mut dur as *mut _,
            0u32,
        );

        // #[repr(C)]
        // struct SpawnRequest {
        //     item_id: u32,
        //     qty: u32,
        //     unk1: u32,
        //     unk2: u32,
        // }

        // type SpawnItemFn = extern "system" fn(*const c_void, *mut SpawnRequest);
        // let spawn_fn_ptr = std::mem::transmute::<_, SpawnItemFn>(self.spawn_item_func_ptr);
        // let pp_map_item_man = self.map_item_man as *const *const c_void;

        // let item_id = self.item_id;
        // let qty = self.qty;

        // let mut spawn_request = SpawnRequest {
        //     item_id,
        //     qty,
        //     unk1: item_id & 0xf0000000,
        //     unk2: 0xffffffff,
        // };

        // spawn_fn_ptr(
        //     *pp_map_item_man,
        //     &mut spawn_request as *mut _,
        // );
    }
}

// 0x00000000 -- Weapon
// 0x10000000 -- Protector
// 0x20000000 -- Accessory
// 0x40000000 -- Goods
// 0x80000000 -- Gem
