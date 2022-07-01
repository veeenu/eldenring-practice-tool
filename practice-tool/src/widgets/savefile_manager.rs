use std::cmp::Ordering;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use imgui::*;
use log::error;

use super::Widget;
use crate::util::{get_key_code, KeyState};

const SFM_TAG: &str = "##savefile-manager";

#[derive(Debug)]
pub(crate) struct ErroredSavefileManagerInner {
    error: String,
}

impl ErroredSavefileManagerInner {
    pub fn new(error: String) -> Self {
        ErroredSavefileManagerInner { error }
    }
}

impl Widget for ErroredSavefileManagerInner {
    fn render(&mut self, ui: &imgui::Ui) {
        ui.text(&self.error);
    }
}

#[derive(Debug)]
pub(crate) struct SavefileManager {
    label: String,
    key_back: KeyState,
    key_close: KeyState,
    key_load: KeyState,
    key_down: KeyState,
    key_up: KeyState,
    key_enter: KeyState,
    dir_stack: DirStack,
    savefile_path: PathBuf,
    breadcrumbs: String,
    savefile_name: String,
    log: Option<String>,

    input_edited: bool,
}

impl SavefileManager {
    pub(crate) fn new_widget(
        key_load: KeyState,
        key_back: KeyState,
        key_close: KeyState,
    ) -> Box<dyn Widget> {
        match SavefileManager::new_inner(key_load, key_back, key_close) {
            Ok(i) => Box::new(i) as _,
            Err(i) => Box::new(i) as _,
        }
    }

    fn new_inner(
        key_load: KeyState,
        key_back: KeyState,
        key_close: KeyState,
    ) -> Result<Self, ErroredSavefileManagerInner> {
        let label = format!("Savefiles (load with {})", key_load);
        let mut savefile_path = get_savefile_path().map_err(|e| {
            ErroredSavefileManagerInner::new(format!("Could not find savefile path: {}", e))
        })?;

        let dir_stack = DirStack::new(&savefile_path).map_err(|e| {
            ErroredSavefileManagerInner::new(format!("Couldn't construct file browser: {}", e))
        })?;

        savefile_path.push("ER0000.sl2");

        Ok(SavefileManager {
            label,
            key_back,
            key_close,
            key_load,
            key_down: KeyState::new(get_key_code("down").unwrap()),
            key_up: KeyState::new(get_key_code("up").unwrap()),
            key_enter: KeyState::new(get_key_code("return").unwrap()),
            dir_stack,
            savefile_path,
            savefile_name: String::new(),
            log: None,
            breadcrumbs: "/".to_string(),
            input_edited: false,
        })
    }

    fn load_savefile(&mut self) {
        if let Some(src_path) = self.dir_stack.current() {
            if src_path.is_file() {
                self.log = match load_savefile(src_path, &self.savefile_path) {
                    Ok(()) => Some(format!(
                        "Loaded {}/{}",
                        if self.breadcrumbs == "/" { "" } else { &self.breadcrumbs },
                        src_path.file_name().unwrap().to_str().unwrap()
                    )),
                    Err(e) => Some(format!("Error loading savefile: {}", e)),
                };
            }
        } else {
            error!("No current path! Can't load savefile.");
        }
    }

    fn import_savefile(&mut self) {
        if self.savefile_name.is_empty() {
            self.log = Some(String::from("Cannot save to empty filename"));
            return;
        }
        if self.savefile_name.contains('/') || self.savefile_name.contains('\\') {
            self.log = Some(String::from("Savefile name cannot contain path separator"));
            return;
        }
        let mut dst_path = PathBuf::from(self.dir_stack.path());
        dst_path.push(&self.savefile_name);
        self.log = match import_savefile(&dst_path, &self.savefile_path) {
            Ok(()) => {
                self.savefile_name.clear();
                self.dir_stack.refresh();
                Some(format!(
                    "Imported {}/{}",
                    if self.breadcrumbs == "/" { "" } else { &self.breadcrumbs },
                    dst_path.file_name().unwrap().to_str().unwrap()
                ))
            },
            Err(e) => Some(format!("Error importing savefile: {}", e)),
        };
    }
}

impl Widget for SavefileManager {
    fn render(&mut self, ui: &imgui::Ui) {
        let scale = super::scaling_factor(ui);
        if ui.button_with_size(&self.label, [super::BUTTON_WIDTH * scale, super::BUTTON_HEIGHT]) {
            ui.open_popup(SFM_TAG);
            self.dir_stack.refresh();
        }

        let style_tokens =
            [ui.push_style_color(imgui::StyleColor::ModalWindowDimBg, super::MODAL_BACKGROUND)];

        if let Some(_token) = PopupModal::new(SFM_TAG)
            .flags(
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::ALWAYS_AUTO_RESIZE,
            )
            .begin_popup(ui)
        {
            ChildWindow::new("##savefile-manager-breadcrumbs").size([240., 14.]).build(ui, || {
                ui.text(&self.breadcrumbs);
                ui.set_scroll_x(ui.scroll_max_x());
            });

            let center_scroll_y = if self.key_down.keyup() {
                self.dir_stack.next();
                true
            } else if self.key_up.keyup() {
                self.dir_stack.prev();
                true
            } else {
                false
            };

            if self.key_enter.keyup() {
                self.dir_stack.enter();
            }

            ListBox::new(SFM_TAG).size([240., 100.]).build(ui, || {
                if Selectable::new(format!(".. Up one dir ({})", self.key_back)).build(ui) {
                    self.dir_stack.exit();
                    self.breadcrumbs = self.dir_stack.breadcrumbs();
                    self.dir_stack.refresh();
                }

                let mut goto: Option<usize> = None;
                for (idx, is_selected, i) in self.dir_stack.values() {
                    if Selectable::new(i).selected(is_selected).build(ui) {
                        goto = Some(idx);
                    }

                    if center_scroll_y && is_selected {
                        ui.set_scroll_here_y();
                    }
                }

                if let Some(idx) = goto {
                    self.dir_stack.goto(idx);
                    self.dir_stack.enter();
                    self.breadcrumbs = self.dir_stack.breadcrumbs();
                }
            });

            if ui.button_with_size(format!("Load savefile ({})", self.key_load), [240., 20.]) {
                self.load_savefile();
            }

            ui.separator();

            {
                let _tok = ui.push_item_width(174.);
                ui.input_text("##savefile_name", &mut self.savefile_name).hint("file name").build();
                self.input_edited = ui.is_item_active();
            }

            ui.same_line();

            if ui.button_with_size("Import", [58., 20.]) {
                self.import_savefile();
            }

            ui.separator();

            if ui.button_with_size("Show folder", [240., 20.]) {
                if let Err(e) = Command::new("explorer.exe")
                    .arg(OsStr::new(self.dir_stack.path().to_str().unwrap()))
                    .spawn()
                {
                    self.log = Some(format!("Couldn't show folder: {}", e));
                };
            }

            if ui.button_with_size(format!("Close ({})", self.key_close), [240., 20.])
                || self.key_close.keyup()
            {
                ui.close_current_popup();
                self.dir_stack.refresh();
            }
        }

        style_tokens.into_iter().rev().for_each(|t| t.pop());
    }

    fn interact(&mut self) {
        if self.input_edited {
            return;
        }
        if self.key_back.keydown() {
            self.dir_stack.exit();
            self.breadcrumbs = self.dir_stack.breadcrumbs();
        } else if self.key_load.keydown() {
            self.load_savefile();
        }
    }

    fn log(&mut self) -> Option<Vec<String>> {
        let log_entry = self.log.take();
        log_entry.map(|e| vec![e])
    }
}

#[derive(Debug)]
struct DirEntry {
    list: Vec<(PathBuf, String)>,
    cursor: usize,
    path: PathBuf,
}

impl DirEntry {
    fn new(path: &Path, cursor: Option<usize>) -> DirEntry {
        let mut list = DirStack::ls(path).unwrap();

        list.sort_by(|a, b| {
            let (ad, bd) = (a.is_dir(), b.is_dir());

            if ad == bd {
                a.cmp(b)
            } else if ad && !bd {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        let list: Vec<_> = list
            .into_iter()
            .map(|a| {
                let repr = if a.is_dir() {
                    format!("+  {}", a.file_name().unwrap().to_str().unwrap())
                } else {
                    format!("   {}", a.file_name().unwrap().to_str().unwrap())
                };
                (a, repr)
            })
            .collect();

        let max_len = list.len();

        DirEntry { list, cursor: cursor.unwrap_or(0).min(max_len), path: PathBuf::from(path) }
    }

    fn values(&self, directories_only: bool) -> impl IntoIterator<Item = (usize, bool, &str)> {
        self.list
            .iter()
            .filter(move |(d, _)| !directories_only || d.is_dir())
            .enumerate()
            .map(|(i, f)| (i, i == self.cursor, f.1.as_str()))
    }

    fn current(&self) -> Option<&PathBuf> {
        self.list.get(self.cursor).as_ref().map(|i| &i.0)
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn goto(&mut self, idx: usize) {
        if idx < self.list.len() {
            self.cursor = idx;
        }
    }

    fn prev(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn next(&mut self) {
        self.cursor = usize::min(self.cursor + 1, self.list.len() - 1);
    }
}

#[derive(Debug)]
struct DirStack {
    top: DirEntry,
    stack: Vec<DirEntry>,
}

impl DirStack {
    fn new(path: &Path) -> Result<Self, String> {
        Ok(DirStack { top: DirEntry::new(path, None), stack: Vec::new() })
    }

    fn enter(&mut self) {
        let new_entry = self
            .stack
            .last()
            .unwrap_or(&self.top)
            .current()
            .filter(|current_entry| current_entry.is_dir())
            .map(|current_entry| DirEntry::new(current_entry, None));

        if let Some(e) = new_entry {
            self.stack.push(e);
        }
    }

    fn exit(&mut self) -> bool {
        if self.stack.is_empty() {
            true
        } else {
            self.stack.pop().unwrap();
            false
        }
    }

    fn breadcrumbs(&self) -> String {
        if self.stack.is_empty() {
            String::from("/")
        } else {
            let mut breadcrumbs = String::new();
            for e in &self.stack {
                breadcrumbs.push('/');
                breadcrumbs.push_str(e.path().file_name().unwrap().to_str().unwrap());
            }
            breadcrumbs
        }
    }

    fn values(&self) -> impl IntoIterator<Item = (usize, bool, &str)> {
        match self.stack.last() {
            Some(d) => d.values(false).into_iter(),
            None => self.top.values(true).into_iter(),
        }
    }

    fn current(&self) -> Option<&PathBuf> {
        self.stack.last().unwrap_or(&self.top).current()
    }

    fn path(&self) -> &PathBuf {
        self.stack.last().unwrap_or(&self.top).path()
    }

    fn goto(&mut self, idx: usize) {
        self.stack.last_mut().unwrap_or(&mut self.top).goto(idx);
    }

    fn prev(&mut self) {
        self.stack.last_mut().unwrap_or(&mut self.top).prev();
    }

    fn next(&mut self) {
        self.stack.last_mut().unwrap_or(&mut self.top).next();
    }

    fn refresh(&mut self) {
        if let Some(l) = self.stack.last_mut() {
            *l = DirEntry::new(l.path(), Some(l.cursor));
        } else {
            self.top = DirEntry::new(self.top.path(), Some(self.top.cursor));
        }
    }

    // TODO SAFETY
    // FS errors would be permission denied (which shouldn't happen but should be
    // reported) and not a directory (which doesn't happen because we checked
    // for is_dir). For the moment, I just unwrap.
    fn ls(path: &Path) -> Result<Vec<PathBuf>, String> {
        Ok(std::fs::read_dir(path)
            .map_err(|e| format!("{}", e))?
            .filter_map(Result::ok)
            .map(|f| f.path())
            .collect())
    }
}

fn get_savefile_path() -> Result<PathBuf, String> {
    let re = regex::Regex::new(r"^[a-f0-9]+$").unwrap();
    let savefile_path: PathBuf =
        [std::env::var("APPDATA").map_err(|e| format!("{}", e))?.as_str(), "EldenRing"]
            .iter()
            .collect();
    std::fs::read_dir(&savefile_path)
        .map_err(|e| format!("{}", e))?
        .filter_map(|e| e.ok())
        .find(|e| re.is_match(&e.file_name().to_string_lossy()) && e.path().is_dir())
        .map(|e| e.path())
        .map(PathBuf::from)
        .ok_or_else(|| String::from("Couldn't find savefile path"))
}

fn load_savefile(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    let buf = std::fs::read(src)?;
    std::fs::write(dest, &buf)?;
    Ok(())
}

fn import_savefile(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    let buf = std::fs::read(dest)?;
    std::fs::write(src, &buf)?;
    Ok(())
}
