use std::ffi::OsString;
use std::fmt::Write;
use std::os::windows::prelude::OsStringExt;
use std::path::PathBuf;
use std::sync::Mutex;

use hudhook::hooks::dx12::ImguiDx12Hooks;
use hudhook::tracing::{error, info};
use hudhook::ImguiRenderLoop;
use imgui::*;
use libeldenring::prelude::*;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use windows::core::PCSTR;
use windows::Win32::Foundation::{GetLastError, HMODULE, MAX_PATH};
use windows::Win32::System::LibraryLoader::{
    GetModuleFileNameW, GetModuleHandleExA, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};

/// Returns the path of the implementor's DLL.
pub fn get_dll_path() -> Option<PathBuf> {
    let mut hmodule = HMODULE(0);
    // SAFETY
    // This is reckless, but it should never fail, and if it does, it's ok to crash
    // and burn.
    if let Err(e) = unsafe {
        GetModuleHandleExA(
            GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT | GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
            PCSTR("DllMain".as_ptr() as _),
            &mut hmodule,
        )
    } {
        error!("get_dll_path: GetModuleHandleExA error: {:x} {e:?}", unsafe { GetLastError().0 },);
        return None;
    }

    let mut sz_filename = [0u16; MAX_PATH as usize];
    // SAFETY
    // pointer to sz_filename always defined and MAX_PATH bounds manually checked
    let len = unsafe { GetModuleFileNameW(hmodule, &mut sz_filename) } as usize;

    Some(OsString::from_wide(&sz_filename[..len]).into())
}

struct ParamTinkerer {
    shown: bool,
    selected_param: usize,
    selected_param_id: usize,
    pointers: Pointers,
}

impl ParamTinkerer {
    fn new() -> Self {
        println!("Initializing");
        hudhook::alloc_console().ok();

        let log_file = get_dll_path()
            .map(|mut path| {
                path.pop();
                path.push("param-tinkerer.log");
                path
            })
            .map(std::fs::File::create);

        match log_file {
            Some(Ok(log_file)) => {
                let file_layer = tracing_subscriber::fmt::layer()
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_thread_names(true)
                    .with_writer(Mutex::new(log_file))
                    .with_ansi(false)
                    .boxed();
                let stdout_layer = tracing_subscriber::fmt::layer()
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_thread_names(true)
                    .with_ansi(true)
                    .boxed();

                tracing_subscriber::registry()
                    .with(LevelFilter::DEBUG)
                    .with(file_layer)
                    .with(stdout_layer)
                    .init();
            },
            e => {
                tracing_subscriber::fmt()
                    .with_max_level(LevelFilter::DEBUG)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_thread_names(true)
                    .with_ansi(true)
                    .init();

                match e {
                    None => error!("Could not construct log file path"),
                    Some(Err(e)) => error!("Could not initialize log file: {:?}", e),
                    _ => unreachable!(),
                }
            },
        }

        ParamTinkerer {
            shown: true,
            selected_param: 0,
            selected_param_id: 0,
            pointers: Pointers::new(),
        }
    }
}

impl ImguiRenderLoop for ParamTinkerer {
    fn render(&mut self, ui: &mut imgui::Ui) {
        if ui.is_key_index_released(0x50) {
            // P key
            self.shown = !self.shown;
            self.pointers.cursor_show.set(self.shown);
        }

        if !self.shown {
            return;
        }

        ui.window("##tool_window")
            .position([16., 16.], Condition::Always)
            .bg_alpha(0.8)
            .flags({
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::ALWAYS_AUTO_RESIZE
            })
            .build(|| {
                let style_tokens =
                    [ui.push_style_color(imgui::StyleColor::ModalWindowDimBg, [0., 0., 0., 0.])];

                self.render_params(ui);

                style_tokens.into_iter().rev().for_each(|t| t.pop());
            });
    }
}

impl ParamTinkerer {
    pub fn render_params(&mut self, ui: &imgui::Ui) {
        let params = PARAMS.write();

        ui.child_window("##param_child_wnd")
            .flags(WindowFlags::NO_SCROLLBAR)
            .size([500., 450.])
            .build(|| {
                const COLUMN1: f32 = 240.;
                const COLUMN2: f32 = 240.;
                const COLUMN3: f32 = 500.;

                ui.columns(2, "##param_columns", false);
                ui.set_column_offset(0, 0.);
                ui.set_column_offset(1, 250.);

                let param_entries = {
                    ui.set_current_column_width(COLUMN1 + 10.);

                    let _ = ui.push_item_width(-1.);
                    ListBox::new("##param_names").size([COLUMN1, 220.]).build(ui, || {
                        for (idx, k) in params.keys().enumerate() {
                            if ui.selectable_config(k).selected(idx == self.selected_param).build()
                            {
                                self.selected_param = idx;
                            }
                        }
                    });

                    params
                        .keys()
                        .nth(self.selected_param)
                        .and_then(|k| unsafe { params.iter_param_ids(k) }.map(|v| (k, v)))
                };

                let param_item = param_entries.map(|(param_name, param_entries)| {
                    ui.next_column();
                    ui.set_current_column_width(COLUMN2 + 10.);

                    let mut buf = String::new();
                    let _ = ui.push_item_width(-1.);
                    ListBox::new("##param_ids").size([COLUMN2, 220.]).build(ui, || {
                        for (idx, id) in param_entries.enumerate() {
                            let param_repr = PARAM_NAMES
                                .get(param_name)
                                .and_then(|param_id_names| param_id_names.get(&(id as usize)))
                                .unwrap_or_else(|| {
                                    buf.clear();
                                    write!(buf, "{}", id).ok();
                                    &buf
                                });
                            if ui
                                .selectable_config(param_repr)
                                .selected(idx == self.selected_param_id)
                                .build()
                            {
                                info!("Selected {idx}: {id}");
                                self.selected_param_id = idx;
                            }
                        }
                    });

                    (param_name, self.selected_param_id)
                });

                if let Some((param_name, param_idx)) = param_item {
                    struct ImguiParamVisitor<'a>(&'a imgui::Ui);

                    impl<'a> ParamVisitor for ImguiParamVisitor<'a> {
                        fn visit_u8(&mut self, name: &str, v: &mut u8) {
                            let mut i = *v as i32;
                            self.0.input_int(name, &mut i).build();
                            *v = i as _;
                        }

                        fn visit_u16(&mut self, name: &str, v: &mut u16) {
                            let mut i = *v as i32;
                            self.0.input_int(name, &mut i).build();
                            *v = i as _;
                        }

                        fn visit_u32(&mut self, name: &str, v: &mut u32) {
                            let mut i = *v as i32;
                            self.0.input_int(name, &mut i).build();
                            *v = i as _;
                        }

                        fn visit_i8(&mut self, name: &str, v: &mut i8) {
                            let mut i = *v as i32;
                            self.0.input_int(name, &mut i).build();
                            *v = i as _;
                        }

                        fn visit_i16(&mut self, name: &str, v: &mut i16) {
                            let mut i = *v as i32;
                            self.0.input_int(name, &mut i).build();
                            *v = i as _;
                        }

                        fn visit_i32(&mut self, name: &str, v: &mut i32) {
                            let mut i = *v;
                            self.0.input_int(name, &mut i).build();
                            *v = i as _;
                        }

                        fn visit_f32(&mut self, name: &str, v: &mut f32) {
                            self.0.input_float(name, v).build();
                        }

                        fn visit_bool(&mut self, name: &str, v: &mut bool) {
                            self.0.checkbox(name, v);
                        }
                    }

                    ui.columns(1, "##param_columns2", false);

                    ListBox::new("##param_detail").size([COLUMN3, 220.]).build(ui, || {
                        let _tok = ui.push_item_width(120.);
                        params.visit_param_item(param_name, param_idx, &mut ImguiParamVisitor(ui));
                    });
                };
            });
    }
}

hudhook::hudhook!(ImguiDx12Hooks, ParamTinkerer::new());
