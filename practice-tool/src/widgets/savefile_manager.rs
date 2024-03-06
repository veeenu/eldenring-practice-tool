use std::path::PathBuf;

use practice_tool_core::key::Key;
use practice_tool_core::widgets::savefile_manager::SavefileManager;
use practice_tool_core::widgets::Widget;

pub(crate) fn savefile_manager(key_load: Option<Key>, key_close: Key) -> Box<dyn Widget> {
    // TODO
    Box::new(SavefileManager::new(key_load, Some(key_close), get_savefile_path().unwrap()))
}

fn get_savefile_path() -> Result<PathBuf, String> {
    let re = regex::Regex::new(r"^[a-f0-9]+$").unwrap();
    let savefile_path: PathBuf =
        [std::env::var("APPDATA").map_err(|e| format!("{}", e))?.as_str(), "EldenRing"]
            .iter()
            .collect();
    std::fs::read_dir(savefile_path)
        .map_err(|e| format!("{}", e))?
        .filter_map(|e| e.ok())
        .find(|e| re.is_match(&e.file_name().to_string_lossy()) && e.path().is_dir())
        .map(|e| e.path())
        .map(|p| p.join("ER0000.sl2"))
        .ok_or_else(|| String::from("Couldn't find savefile path"))
}

// use std::cmp::Ordering;
// use std::path::{Path, PathBuf};
// use std::process::{Command, Stdio};
//
// use hudhook::tracing::error;
// use imgui::sys::{igGetCursorPosX, igGetCursorPosY, igGetWindowPos,
// igSetNextWindowPos, ImVec2}; use imgui::*;
//
// use super::{scaling_factor, Widget, BUTTON_HEIGHT, BUTTON_WIDTH};
// use crate::util::{get_key_code, KeyState};
//
// const SFM_TAG: &str = "##savefile-manager";
// const SFML_TAG: &str = "##savefile-manager-list";
//
// #[derive(Debug)]
// pub(crate) struct ErroredSavefileManagerInner {
//     error: String,
// }
//
// impl ErroredSavefileManagerInner {
//     pub fn new(error: String) -> Self {
//         ErroredSavefileManagerInner { error }
//     }
// }
//
// impl Widget for ErroredSavefileManagerInner {
//     fn render(&mut self, ui: &imgui::Ui) {
//         ui.text(&self.error);
//     }
// }
//
// #[derive(Debug)]
// pub(crate) struct SavefileManager {
//     label: String,
//     label_close: String,
//     label_load: String,
//     hotkey_load: KeyState,
//     hotkey_down: KeyState,
//     hotkey_up: KeyState,
//     hotkey_enter: KeyState,
//     hotkey_open: Option<KeyState>,
//     hotkey_close: KeyState,
//     dir_stack: DirStack,
//     savefile_path: PathBuf,
//     breadcrumbs: String,
//     savefile_name: String,
//     log: Option<String>,
//
//     input_edited: bool,
// }
//
// impl SavefileManager {
//     pub(crate) fn new_widget(
//         hotkey_load: KeyState,
//         hotkey_open: Option<KeyState>,
//         hotkey_close: KeyState,
//     ) -> Box<dyn Widget> {
//         match SavefileManager::new_inner(hotkey_load, hotkey_open,
// hotkey_close) {             Ok(i) => Box::new(i) as _,
//             Err(i) => Box::new(i) as _,
//         }
//     }
//
//     fn new_inner(
//         hotkey_load: KeyState,
//         hotkey_open: Option<KeyState>,
//         hotkey_close: KeyState,
//     ) -> Result<Self, ErroredSavefileManagerInner> {
//         let label = format!("Savefiles (load with {hotkey_load})");
//         let label_load = format!("Load savefile ({hotkey_load})");
//         let label_close = format!("Close ({})", hotkey_close);
//
//         let mut savefile_path = get_savefile_path().map_err(|e| {
//             ErroredSavefileManagerInner::new(format!("Could not find savefile
// path: {}", e))         })?;
//
//         let dir_stack = DirStack::new(&savefile_path).map_err(|e| {
//             ErroredSavefileManagerInner::new(format!("Couldn't construct file
// browser: {}", e))         })?;
//
//         savefile_path.push("ER0000.sl2");
//
//         Ok(SavefileManager {
//             label,
//             label_load,
//             label_close,
//             hotkey_load,
//             hotkey_down: KeyState::new(get_key_code("down").unwrap(), None),
//             hotkey_up: KeyState::new(get_key_code("up").unwrap(), None),
//             hotkey_enter: KeyState::new(get_key_code("return").unwrap(),
// None),             hotkey_open,
//             hotkey_close,
//             dir_stack,
//             savefile_path,
//             savefile_name: String::new(),
//             log: None,
//             breadcrumbs: "/".to_string(),
//             input_edited: false,
//         })
//     }
//
//     fn is_opening(&self, ui: &imgui::Ui) -> bool {
//         self.hotkey_open.map(|k| k.keydown(ui)).unwrap_or(false)
//     }
//
//     fn load_savefile(&mut self) {
//         if let Some(src_path) = self.dir_stack.current() {
//             if src_path.is_file() {
//                 self.log = match load_savefile(src_path, &self.savefile_path)
// {                     Ok(()) => Some(format!(
//                         "Loaded {}/{}",
//                         if self.breadcrumbs == "/" { "" } else {
// &self.breadcrumbs },
// src_path.file_name().unwrap().to_str().unwrap()                     )),
//                     Err(e) => Some(format!("Error loading savefile: {}", e)),
//                 };
//             }
//         } else {
//             error!("No current path! Can't load savefile.");
//         }
//     }
//
//     fn import_savefile(&mut self) {
//         if self.savefile_name.is_empty() {
//             self.log = Some(String::from("Cannot save to empty filename"));
//             return;
//         }
//         if self.savefile_name.contains('/') ||
// self.savefile_name.contains('\\') {             self.log =
// Some(String::from("Savefile name cannot contain path separator"));
//             return;
//         }
//         let mut dst_path = PathBuf::from(self.dir_stack.path());
//         dst_path.push(&self.savefile_name);
//         self.log = match import_savefile(&dst_path, &self.savefile_path) {
//             Ok(()) => {
//                 self.savefile_name.clear();
//                 self.dir_stack.refresh();
//                 Some(format!(
//                     "Imported {}/{}",
//                     if self.breadcrumbs == "/" { "" } else {
// &self.breadcrumbs },
// dst_path.file_name().unwrap().to_str().unwrap()                 ))
//             },
//             Err(e) => Some(format!("Error importing savefile: {}", e)),
//         };
//     }
// }
//
// impl Widget for SavefileManager {
//     fn render(&mut self, ui: &imgui::Ui) {
//         let scale = scaling_factor(ui);
//         let button_width = BUTTON_WIDTH * scale;
//
//         let (x, y) = unsafe {
//             let mut wnd_pos = ImVec2::default();
//             igGetWindowPos(&mut wnd_pos);
//             (igGetCursorPosX() + wnd_pos.x, igGetCursorPosY() + wnd_pos.y)
//         };
//
//         if ui.button_with_size(&self.label, [button_width, BUTTON_HEIGHT]) ||
// self.is_opening(ui) {             ui.open_popup(SFM_TAG);
//             self.dir_stack.refresh();
//         }
//
//         unsafe {
//             igSetNextWindowPos(
//                 ImVec2::new(x + 200. * scale, y),
//                 Condition::Always as i8 as _,
//                 ImVec2::new(0., 0.),
//             )
//         };
//
//         if let Some(_token) = ui
//             .modal_popup_config(SFM_TAG)
//             .resizable(false)
//             .movable(false)
//             .title_bar(false)
//             .scroll_bar(false)
//             .begin_popup()
//         {
//             ui.child_window("##savefile-manager-breadcrumbs")
//                 .size([button_width, 20. * scale])
//                 .build(|| {
//                     ui.text(&self.breadcrumbs);
//                     ui.set_scroll_x(ui.scroll_max_x());
//                 });
//
//             let center_scroll_y = if self.hotkey_down.keyup(ui) {
//                 self.dir_stack.next();
//                 true
//             } else if self.hotkey_up.keyup(ui) {
//                 self.dir_stack.prev();
//                 true
//             } else {
//                 false
//             };
//
//             if self.hotkey_enter.keyup(ui) {
//                 self.dir_stack.enter();
//             }
//
//             ListBox::new(SFML_TAG).size([button_width, 200. *
// scale]).build(ui, || {                 if ui.selectable_config(".. Up one
// dir").build() {                     self.dir_stack.exit();
//                     self.breadcrumbs = self.dir_stack.breadcrumbs();
//                     self.dir_stack.refresh();
//                 }
//
//                 let mut goto: Option<usize> = None;
//                 for (idx, is_selected, i) in self.dir_stack.values() {
//                     if ui.selectable_config(i).selected(is_selected).build()
// {                         goto = Some(idx);
//                     }
//
//                     if center_scroll_y && is_selected {
//                         ui.set_scroll_here_y();
//                     }
//                 }
//
//                 if let Some(idx) = goto {
//                     self.dir_stack.goto(idx);
//                     self.dir_stack.enter();
//                     self.breadcrumbs = self.dir_stack.breadcrumbs();
//                 }
//             });
//
//             if ui.button_with_size(&self.label_load, [button_width,
// BUTTON_HEIGHT]) {                 self.load_savefile();
//             }
//
//             ui.separator();
//
//             {
//                 let _tok = ui.push_item_width(button_width * 174. / 240.);
//                 ui.input_text("##savefile_name", &mut
// self.savefile_name).hint("file name").build();
// self.input_edited = ui.is_item_active();             }
//
//             ui.same_line();
//
//             if ui.button_with_size("Import", [button_width * 58. / 240.,
// BUTTON_HEIGHT]) {                 self.import_savefile();
//             }
//
//             ui.separator();
//
//             if ui.button_with_size("Show folder", [button_width,
// BUTTON_HEIGHT]) {                 let path =
// self.dir_stack.path().to_owned();                 let path = if path.is_dir()
// { &path } else { path.parent().unwrap() };
//
//                 if let Err(e) = Command::new("explorer.exe")
//                     .stdin(Stdio::null())
//                     .stdout(Stdio::null())
//                     .stderr(Stdio::null())
//                     .arg(path.as_os_str())
//                     .spawn()
//                 {
//                     self.log = Some(format!("Couldn't show folder: {}", e));
//                 };
//             }
//
//             if ui.button_with_size(&self.label_close, [button_width,
// BUTTON_HEIGHT])                 || (self.hotkey_close.keyup(ui) &&
// !ui.is_any_item_active())             {
//                 ui.close_current_popup();
//                 self.dir_stack.refresh();
//             }
//         }
//     }
//
//     fn interact(&mut self, ui: &imgui::Ui) {
//         if self.input_edited {
//             return;
//         }
//
//         if !ui.is_any_item_active() && self.hotkey_load.keydown(ui) &&
// !self.is_opening(ui) {             self.load_savefile();
//         }
//     }
//
//     fn log(&mut self) -> Option<Vec<String>> {
//         let log_entry = self.log.take();
//         log_entry.map(|e| vec![e])
//     }
// }
//
// #[derive(Debug)]
// struct DirEntry {
//     list: Vec<(PathBuf, String)>,
//     cursor: usize,
//     path: PathBuf,
// }
//
// impl DirEntry {
//     fn new(path: &Path, cursor: Option<usize>) -> DirEntry {
//         let mut list = DirStack::ls(path).unwrap();
//
//         list.sort_by(|a, b| {
//             let (ad, bd) = (a.is_dir(), b.is_dir());
//
//             if ad == bd {
//                 a.cmp(b)
//             } else if ad && !bd {
//                 Ordering::Less
//             } else {
//                 Ordering::Greater
//             }
//         });
//
//         let list: Vec<_> = list
//             .into_iter()
//             .map(|a| {
//                 let repr = if a.is_dir() {
//                     format!("+  {}",
// a.file_name().unwrap().to_str().unwrap())                 } else {
//                     format!("   {}",
// a.file_name().unwrap().to_str().unwrap())                 };
//                 (a, repr)
//             })
//             .collect();
//
//         let max_len = list.len();
//
//         DirEntry { list, cursor: cursor.unwrap_or(0).min(max_len), path:
// PathBuf::from(path) }     }
//
//     fn values(&self, directories_only: bool) -> impl IntoIterator<Item =
// (usize, bool, &str)> {         self.list
//             .iter()
//             .filter(move |(d, _)| !directories_only || d.is_dir())
//             .enumerate()
//             .map(|(i, f)| (i, i == self.cursor, f.1.as_str()))
//     }
//
//     fn current(&self) -> Option<&PathBuf> {
//         self.list.get(self.cursor).as_ref().map(|i| &i.0)
//     }
//
//     fn path(&self) -> &PathBuf {
//         &self.path
//     }
//
//     fn goto(&mut self, idx: usize) {
//         if idx < self.list.len() {
//             self.cursor = idx;
//         }
//     }
//
//     fn prev(&mut self) {
//         self.cursor = self.cursor.saturating_sub(1);
//     }
//
//     fn next(&mut self) {
//         self.cursor = usize::min(self.cursor + 1, self.list.len() - 1);
//     }
// }
//
// #[derive(Debug)]
// struct DirStack {
//     top: DirEntry,
//     stack: Vec<DirEntry>,
// }
//
// impl DirStack {
//     fn new(path: &Path) -> Result<Self, String> {
//         Ok(DirStack { top: DirEntry::new(path, None), stack: Vec::new() })
//     }
//
//     fn enter(&mut self) {
//         let new_entry = self
//             .stack
//             .last()
//             .unwrap_or(&self.top)
//             .current()
//             .filter(|current_entry| current_entry.is_dir())
//             .map(|current_entry| DirEntry::new(current_entry, None));
//
//         if let Some(e) = new_entry {
//             self.stack.push(e);
//         }
//     }
//
//     fn exit(&mut self) -> bool {
//         if self.stack.is_empty() {
//             true
//         } else {
//             self.stack.pop().unwrap();
//             false
//         }
//     }
//
//     fn breadcrumbs(&self) -> String {
//         if self.stack.is_empty() {
//             String::from("/")
//         } else {
//             let mut breadcrumbs = String::new();
//             for e in &self.stack {
//                 breadcrumbs.push('/');
//
// breadcrumbs.push_str(e.path().file_name().unwrap().to_str().unwrap());
//             }
//             breadcrumbs
//         }
//     }
//
//     fn values(&self) -> impl IntoIterator<Item = (usize, bool, &str)> {
//         match self.stack.last() {
//             Some(d) => d.values(false).into_iter(),
//             None => self.top.values(true).into_iter(),
//         }
//     }
//
//     fn current(&self) -> Option<&PathBuf> {
//         self.stack.last().unwrap_or(&self.top).current()
//     }
//
//     fn path(&self) -> &PathBuf {
//         self.stack.last().unwrap_or(&self.top).path()
//     }
//
//     fn goto(&mut self, idx: usize) {
//         self.stack.last_mut().unwrap_or(&mut self.top).goto(idx);
//     }
//
//     fn prev(&mut self) {
//         self.stack.last_mut().unwrap_or(&mut self.top).prev();
//     }
//
//     fn next(&mut self) {
//         self.stack.last_mut().unwrap_or(&mut self.top).next();
//     }
//
//     fn refresh(&mut self) {
//         if let Some(l) = self.stack.last_mut() {
//             *l = DirEntry::new(l.path(), Some(l.cursor));
//         } else {
//             self.top = DirEntry::new(self.top.path(), Some(self.top.cursor));
//         }
//     }
//
//     // TODO SAFETY
//     // FS errors would be permission denied (which shouldn't happen but
// should be     // reported) and not a directory (which doesn't happen because
// we checked     // for is_dir). For the moment, I just unwrap.
//     fn ls(path: &Path) -> Result<Vec<PathBuf>, String> {
//         Ok(std::fs::read_dir(path)
//             .map_err(|e| format!("{}", e))?
//             .filter_map(Result::ok)
//             .map(|f| f.path())
//             .collect())
//     }
// }
//
// fn get_savefile_path() -> Result<PathBuf, String> {
//     let re = regex::Regex::new(r"^[a-f0-9]+$").unwrap();
//     let savefile_path: PathBuf =
//         [std::env::var("APPDATA").map_err(|e| format!("{}", e))?.as_str(),
// "EldenRing"]             .iter()
//             .collect();
//     std::fs::read_dir(savefile_path)
//         .map_err(|e| format!("{}", e))?
//         .filter_map(|e| e.ok())
//         .find(|e| re.is_match(&e.file_name().to_string_lossy()) &&
// e.path().is_dir())         .map(|e| e.path())
//         .map(PathBuf::from)
//         .ok_or_else(|| String::from("Couldn't find savefile path"))
// }
//
// fn load_savefile(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
//     let buf = std::fs::read(src)?;
//     std::fs::write(dest, buf)?;
//     Ok(())
// }
//
// fn import_savefile(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
//     let buf = std::fs::read(dest)?;
//     std::fs::write(src, buf)?;
//     Ok(())
// }
