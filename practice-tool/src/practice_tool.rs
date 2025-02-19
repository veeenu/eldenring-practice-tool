use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use const_format::formatcp;
use hudhook::tracing::metadata::LevelFilter;
use hudhook::tracing::*;
use hudhook::{ImguiRenderLoop, RenderContext};
use imgui::*;
use libeldenring::prelude::*;
use libeldenring::version;
use pkg_version::*;
use practice_tool_core::crossbeam_channel::{self, Receiver, Sender};
use practice_tool_core::widgets::radial_menu::radial_menu;
use practice_tool_core::widgets::{scaling_factor, Widget, BUTTON_HEIGHT, BUTTON_WIDTH};
use sys::ImVec2;
use tracing_subscriber::prelude::*;
use windows::Win32::UI::Input::XboxController::{XINPUT_GAMEPAD_A, XINPUT_GAMEPAD_B, XINPUT_STATE};

use crate::config::{Config, IndicatorType, RadialMenu, Settings};
use crate::update::Update;
use crate::{util, XINPUTGETSTATE};

const MAJOR: usize = pkg_version_major!();
const MINOR: usize = pkg_version_minor!();
const PATCH: usize = pkg_version_patch!();

pub(crate) static BLOCK_XINPUT: AtomicBool = AtomicBool::new(false);

struct FontIDs {
    small: FontId,
    normal: FontId,
    big: FontId,
}

unsafe impl Send for FontIDs {}
unsafe impl Sync for FontIDs {}

enum UiState {
    MenuOpen,
    Closed,
    Hidden,
}

pub(crate) struct PracticeTool {
    settings: Settings,
    pointers: Pointers,
    version_label: String,
    widgets: Vec<Box<dyn Widget>>,
    radial_menu: Vec<RadialMenu>,

    log: Vec<(Instant, String)>,
    log_rx: Receiver<String>,
    log_tx: Sender<String>,
    ui_state: UiState,
    fonts: Option<FontIDs>,
    config_err: Option<String>,
    update_available: Update,

    position_bufs: [String; 4],
    position_prev: [f32; 3],
    position_change_buf: String,

    position_dist_ref: [f32; 3],
    position_dist_buf: String,

    igt_buf: String,
    fps_buf: String,

    framecount: u32,
    framecount_buf: String,

    cur_anim_buf: String,

    gamepad_state: XINPUT_STATE,
    gamepad_stick: ImVec2,
    radial_menu_open_time: Instant,
    press_queue: Vec<imgui::Key>,
    release_queue: Vec<imgui::Key>,
}

impl PracticeTool {
    pub(crate) fn new() -> Self {
        hudhook::alloc_console().ok();
        log_panics::init();

        fn load_config() -> Result<Config, String> {
            let config_path = crate::util::get_dll_path()
                .map(|mut path| {
                    path.pop();
                    path.push("jdsd_er_practice_tool.toml");
                    path
                })
                .ok_or_else(|| "Couldn't find config file".to_string())?;

            if !config_path.exists() {
                std::fs::write(&config_path, include_str!("../../jdsd_er_practice_tool.toml"))
                    .map_err(|e| format!("Couldn't write default config file: {}", e))?;
            }

            let config_content = std::fs::read_to_string(config_path)
                .map_err(|e| format!("Couldn't read config file: {}", e))?;
            println!("{}", config_content);
            Config::parse(&config_content).map_err(String::from)
        }

        let (config, config_err) = match load_config() {
            Ok(config) => (config, None),
            Err(e) => (
                Config::default(),
                Some({
                    error!("{}", e);
                    format!(
                        "Configuration error, please review your jdsd_er_practice_tool.toml \
                         file.\n\n{e}"
                    )
                }),
            ),
        };

        let log_file = util::get_dll_path()
            .map(|mut path| {
                path.pop();
                path.push("jdsd_er_practice_tool.log");
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
                    .with(config.settings.log_level.inner())
                    .with(file_layer)
                    .with(stdout_layer)
                    .init();
            },
            e => match e {
                None => error!("Could not construct log file path"),
                Some(Err(e)) => error!("Could not initialize log file: {:?}", e),
                _ => unreachable!(),
            },
        }

        if config.settings.dxgi_debug {
            hudhook::util::enable_debug_interface();
        }

        if config.settings.log_level.inner() < LevelFilter::DEBUG || !config.settings.show_console {
            hudhook::free_console().ok();
        }

        let pointers = Pointers::new();
        let poll_interval = Duration::from_millis(100);
        loop {
            if let Some(menu_timer) = pointers.menu_timer.read() {
                if menu_timer > 0. {
                    break;
                }
            }
            thread::sleep(poll_interval);
        }
        wait_option_thread(
            || unsafe {
                let mut params = PARAMS.write();
                if let Err(e) = params.refresh() {
                    error!("{}", e);
                }
                params.get_equip_param_goods()
            },
            |mut epg| {
                if let Some(spectral_steed_whistle) =
                    epg.find(|i| i.id == 130).and_then(|p| p.param)
                {
                    spectral_steed_whistle.icon_id = 12;
                };
            },
        );

        let update_available =
            if config.settings.disable_update_prompt { Update::UpToDate } else { Update::check() };

        let version_label = {
            let (maj, min, patch) = version::get_version().into();
            format!("Game Ver {}.{:02}.{}", maj, min, patch)
        };
        let settings = config.settings.clone();
        let radial_menu = config.radial_menu.clone();
        let widgets = config.make_commands(&pointers);

        let (log_tx, log_rx) = crossbeam_channel::unbounded();
        info!("Practice tool initialized");

        PracticeTool {
            settings,
            pointers,
            version_label,
            widgets,
            log: Vec::new(),
            log_rx,
            log_tx,
            fonts: None,
            ui_state: UiState::Closed,
            config_err,
            position_prev: Default::default(),
            position_bufs: Default::default(),
            position_change_buf: Default::default(),
            position_dist_ref: Default::default(),
            position_dist_buf: Default::default(),
            igt_buf: Default::default(),
            fps_buf: Default::default(),
            framecount: 0,
            framecount_buf: Default::default(),
            cur_anim_buf: Default::default(),
            update_available,
            radial_menu,
            gamepad_state: Default::default(),
            gamepad_stick: Default::default(),
            radial_menu_open_time: Instant::now(),
            press_queue: Vec::new(),
            release_queue: Vec::new(),
        }
    }

    fn render_visible(&mut self, ui: &imgui::Ui) {
        let [dw, dh] = { ui.io().display_size };
        ui.window("##tool_window")
            .position([16., 16.], Condition::Always)
            .size_constraints([240., 0.], [dw - 70., dh - 70.])
            .bg_alpha(0.8)
            .flags({
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::ALWAYS_AUTO_RESIZE
            })
            .build(|| {
                if let Some(e) = self.config_err.as_ref() {
                    ui.text(e);
                }

                if !(ui.io().want_capture_keyboard && ui.is_any_item_active()) {
                    for w in self.widgets.iter_mut() {
                        w.interact(ui);
                    }
                }

                for w in self.widgets.iter_mut() {
                    w.render(ui);
                }

                if ui.button_with_size("Close", [BUTTON_WIDTH * scaling_factor(ui), BUTTON_HEIGHT])
                {
                    self.ui_state = UiState::Closed;
                    self.pointers.cursor_show.set(false);
                }

                if option_env!("CARGO_XTASK_DIST").is_none()
                    && ui.button_with_size("Eject", [
                        BUTTON_WIDTH * scaling_factor(ui),
                        BUTTON_HEIGHT,
                    ])
                {
                    self.ui_state = UiState::Closed;
                    self.pointers.cursor_show.set(false);
                    hudhook::eject();
                }
            });
    }

    fn render_closed(&mut self, ui: &imgui::Ui) {
        let [w, h] = ui.io().display_size;

        let stack_tokens = vec![
            ui.push_style_var(StyleVar::WindowRounding(0.)),
            ui.push_style_var(StyleVar::FrameBorderSize(0.)),
            ui.push_style_var(StyleVar::WindowBorderSize(0.)),
        ];
        ui.window("##msg_window")
            .position([w * 35. / 1920., h * 140. / 1080.], Condition::Always)
            .bg_alpha(0.0)
            .flags({
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::ALWAYS_AUTO_RESIZE
            })
            .build(|| {
                ui.text("johndisandonato's Practice Tool");

                // ui.same_line();

                if ui.small_button("Open") {
                    self.ui_state = UiState::MenuOpen;
                }

                ui.same_line();

                if ui.small_button("Indicators") {
                    ui.open_popup("##indicators_window");
                }

                ui.modal_popup_config("##indicators_window")
                    .resizable(false)
                    .movable(false)
                    .title_bar(false)
                    .build(|| {
                        let style = ui.clone_style();

                        self.pointers.cursor_show.set(true);

                        ui.text(
                            "You can toggle indicators here, as\nwell as reset the frame \
                             counter.\n\nKeep in mind that the available\nindicators and order of \
                             them depend\non your config file.",
                        );
                        ui.separator();

                        for indicator in &mut self.settings.indicators {
                            let label = match indicator.indicator {
                                IndicatorType::GameVersion => "Game Version",
                                IndicatorType::Position => "Player Position",
                                IndicatorType::PositionChange => "Player Velocity",
                                IndicatorType::PositionDistance => "Player Distance",
                                IndicatorType::Animation => "Animation",
                                IndicatorType::Igt => "IGT Timer",
                                IndicatorType::Fps => "FPS",
                                IndicatorType::FrameCount => "Frame Counter",
                                IndicatorType::ImguiDebug => "ImGui Debug Info",
                            };

                            let mut state = indicator.enabled;

                            if ui.checkbox(label, &mut state) {
                                indicator.enabled = state;
                            }

                            if let IndicatorType::FrameCount = indicator.indicator {
                                ui.same_line();

                                let btn_reset_label = "Reset";
                                let btn_reset_width = ui.calc_text_size(btn_reset_label)[0]
                                    + style.frame_padding[0] * 2.0;

                                ui.set_cursor_pos([
                                    ui.content_region_max()[0] - btn_reset_width,
                                    ui.cursor_pos()[1],
                                ]);

                                if ui.button("Reset") {
                                    self.framecount = 0;
                                }
                            }

                            if let IndicatorType::PositionDistance = indicator.indicator {
                                ui.same_line();

                                let btn_xyz_label = "Start XYZ";
                                let btn_xyz_width = ui.calc_text_size(btn_xyz_label)[0]
                                    + style.frame_padding[0] * 2.0;

                                ui.set_cursor_pos([
                                    ui.content_region_max()[0] - btn_xyz_width,
                                    ui.cursor_pos()[1],
                                ]);

                                if ui.button("Start XYZ") {
                                    if let Some([x, y, z, _a1, _a2]) = self.pointers.global_position.read() {
                                        self.position_dist_ref = [x, y, z];
                                    }
                                }
                            }
                        }

                        ui.separator();

                        let btn_close_width =
                            ui.content_region_max()[0] - style.frame_padding[0] * 2.0;

                        if ui.button_with_size("Close", [btn_close_width, 0.0]) {
                            ui.close_current_popup();
                            self.pointers.cursor_show.set(false);
                        }
                    });

                ui.same_line();

                if ui.small_button("Help") {
                    ui.open_popup("##help_window");
                }

                match &self.update_available {
                    Update::UpToDate => {},
                    Update::Available { .. } => {
                        ui.same_line();

                        let green = [0.1, 0.7, 0.1, 1.0];
                        let _token = ui.push_style_color(StyleColor::Button, green);

                        if ui.small_button("Update") {
                            ui.open_popup("##update");
                        }
                    },
                    Update::Error(_) => {
                        ui.same_line();

                        let red = [1.0, 0.0, 0.0, 1.0];
                        let _token = ui.push_style_color(StyleColor::Button, red);

                        if ui.small_button("Update") {
                            ui.open_popup("##update");
                        }
                    },
                }

                ui.modal_popup_config("##help_window")
                    .resizable(false)
                    .movable(false)
                    .title_bar(false)
                    .build(|| {
                        self.pointers.cursor_show.set(true);
                        ui.text(formatcp!(
                            "Elden Ring Practice Tool v{}.{}.{}",
                            MAJOR,
                            MINOR,
                            PATCH
                        ));
                        ui.separator();
                        ui.text(format!(
                            "Press the {} key to open/close the tool's\ninterface.\n\nYou can \
                             toggle flags/launch commands by\nclicking in the UI or by \
                             pressing\nthe hotkeys (in the parentheses).\n\nYou can configure \
                             your tool by editing\nthe jdsd_er_practice_tool.toml file with\na \
                             text editor. If you break something,\njust download a fresh \
                             file!\n\nThank you for using my tool! <3\n",
                            self.settings.display
                        ));
                        ui.separator();
                        ui.text("-- johndisandonato");
                        ui.text("   https://twitch.tv/johndisandonato");
                        if ui.is_item_clicked() {
                            open::that("https://twitch.tv/johndisandonato").ok();
                        }
                        ui.separator();
                        if ui.button("Submit issue") {
                            open::that(
                                "https://github.com/veeenu/eldenring-practice-tool/issues/new",
                            )
                            .ok();
                        }
                        ui.same_line();
                        if ui.button("Support") {
                            open::that("https://patreon.com/johndisandonato").ok();
                        }
                        ui.same_line();
                        if ui.button("Close") {
                            ui.close_current_popup();
                            self.pointers.cursor_show.set(false);
                        }
                    });

                ui.modal_popup_config("##update")
                    .resizable(false)
                    .movable(false)
                    .title_bar(false)
                    .build(|| {
                        self.pointers.cursor_show.set(true);

                        match &self.update_available {
                            Update::UpToDate => {
                                ui.close_current_popup();
                            },
                            Update::Available { url, notes } => {
                                ui.text(notes);
                                if ui.button("Download") {
                                    open::that(url).ok();
                                }
                                ui.same_line();
                            },
                            Update::Error(e) => {
                                ui.text("Update error: could not check for updates.");
                                ui.separator();
                                ui.text(e);
                            },
                        }

                        if ui.button("Close") {
                            ui.close_current_popup();
                            self.pointers.cursor_show.set(false);
                        }
                    });

                ui.new_line();

                for indicator in &self.settings.indicators {
                    if !indicator.enabled {
                        continue;
                    }

                    match indicator.indicator {
                        IndicatorType::GameVersion => {
                            ui.text(&self.version_label);
                        },
                        IndicatorType::Position => {
                            if let (Some([x, y, z, _a1, _a2]), Some(m)) = (
                                self.pointers.global_position.read(),
                                self.pointers.global_position.read_map_id(),
                            ) {
                                let (a, b, r, s) =
                                    ((m >> 24) & 0xff, (m >> 16) & 0xff, (m >> 8) & 0xff, m & 0xff);
                                self.position_bufs.iter_mut().for_each(String::clear);
                                write!(self.position_bufs[0], "m{a:02x}_{b:02x}_{r:02x}_{s:02x}")
                                    .ok();
                                write!(self.position_bufs[1], "{x:.3}").ok();
                                write!(self.position_bufs[2], "{y:.3}").ok();
                                write!(self.position_bufs[3], "{z:.3}").ok();

                                ui.text(&self.position_bufs[0]);
                                ui.same_line();
                                ui.text_colored(
                                    [0.7048, 0.1228, 0.1734, 1.],
                                    &self.position_bufs[1],
                                );
                                ui.same_line();
                                ui.text_colored(
                                    [0.1161, 0.5327, 0.3512, 1.],
                                    &self.position_bufs[2],
                                );
                                ui.same_line();
                                ui.text_colored(
                                    [0.1445, 0.2852, 0.5703, 1.],
                                    &self.position_bufs[3],
                                );
                            }
                        },
                        IndicatorType::PositionChange => {
                            if let Some([x, y, z, _a1, _a2]) = self.pointers.global_position.read()
                            {
                                let position_change_xyz = ((x - self.position_prev[0]).powf(2.0)
                                    + (y - self.position_prev[1]).powf(2.0)
                                    + (z - self.position_prev[2]).powf(2.0))
                                .sqrt();

                                let position_change_xz = ((x - self.position_prev[0]).powf(2.0)
                                    + (z - self.position_prev[2]).powf(2.0))
                                .sqrt();

                                let position_change_y = y - self.position_prev[1];

                                self.position_change_buf.clear();
                                write!(
                                    self.position_change_buf,
                                    "Velocity: [XYZ] {position_change_xyz:.3} | [XZ] \
                                     {position_change_xz:.3} | [Y] {position_change_y:.3}"
                                )
                                .ok();
                                ui.text(&self.position_change_buf);

                                self.position_prev = [x, y, z];
                            }
                        },
                        IndicatorType::PositionDistance => {
                            if let Some([x, y, z, _a1, _a2]) = self.pointers.global_position.read()
                            {
                                let position_dist_xyz = ((x - self.position_dist_ref[0]).powf(2.0)
                                    + (y - self.position_dist_ref[1]).powf(2.0)
                                    + (z - self.position_dist_ref[2]).powf(2.0))
                                .sqrt();

                                let position_dist_xz = ((x - self.position_dist_ref[0]).powf(2.0)
                                    + (z - self.position_dist_ref[2]).powf(2.0))
                                .sqrt();

                                let position_dist_y = y - self.position_dist_ref[1];

                                self.position_dist_buf.clear();
                                write!(
                                    self.position_dist_buf,
                                    "Distance: [XYZ] {position_dist_xyz:.4} | [XZ] \
                                     {position_dist_xz:.4} | [Y] {position_dist_y:.4}"
                                )
                                .ok();
                                ui.text(&self.position_dist_buf);
                            }
                        },
                        IndicatorType::Animation => {
                            if let (Some(cur_anim), Some(cur_anim_time), Some(cur_anim_length)) = (
                                self.pointers.cur_anim.read(),
                                self.pointers.cur_anim_time.read(),
                                self.pointers.cur_anim_length.read(),
                            ) {
                                self.cur_anim_buf.clear();
                                write!(
                                    self.cur_anim_buf,
                                    "Animation {cur_anim} ({cur_anim_time}s /  {cur_anim_length}s)",
                                )
                                .ok();
                                ui.text(&self.cur_anim_buf);
                            }
                        },
                        IndicatorType::Igt => {
                            if let Some(igt) = self.pointers.igt.read() {
                                let millis = (igt % 1000) / 10;
                                let total_seconds = igt / 1000;
                                let seconds = total_seconds % 60;
                                let minutes = total_seconds / 60 % 60;
                                let hours = total_seconds / 3600;
                                self.igt_buf.clear();
                                write!(
                                    self.igt_buf,
                                    "IGT {hours:02}:{minutes:02}:{seconds:02}.{millis:02}",
                                )
                                .ok();
                                ui.text(&self.igt_buf);
                            }
                        },
                        IndicatorType::Fps => {
                            if let Some(fps) = self.pointers.fps.read() {
                                self.fps_buf.clear();
                                write!(self.fps_buf, "FPS {fps}",).ok();
                                ui.text(&self.fps_buf);
                            }
                        },
                        IndicatorType::FrameCount => {
                            self.framecount_buf.clear();
                            write!(self.framecount_buf, "Frame count {0}", self.framecount,).ok();
                            ui.text(&self.framecount_buf);
                        },
                        IndicatorType::ImguiDebug => {
                            imgui_debug(ui);
                        },
                    }
                }

                for w in self.widgets.iter_mut() {
                    w.render_closed(ui);
                }

                for w in self.widgets.iter_mut() {
                    w.interact(ui);
                }
            });

        for st in stack_tokens.into_iter().rev() {
            st.pop();
        }
    }

    fn render_hidden(&mut self, ui: &imgui::Ui) {
        for w in self.widgets.iter_mut() {
            w.interact(ui);
        }
    }

    fn render_radial(&mut self, ui: &imgui::Ui) {
        // Debounce a handful of frames to avoid accidentally rotating the menu when
        // releasing L3
        const RADIAL_MENU_DEBOUNCE: Duration = Duration::from_millis(150);

        let Some(combo) = self.settings.radial_menu_open.as_ref() else {
            return;
        };

        let pressed_a_before = self.gamepad_state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_A);
        let pressed_b_before = self.gamepad_state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_B);

        let [_, h] = ui.io().display_size;
        unsafe { (XINPUTGETSTATE)(0, &mut self.gamepad_state) };

        let pressed_a_after = self.gamepad_state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_A);
        let pressed_b_after = self.gamepad_state.Gamepad.wButtons.contains(XINPUT_GAMEPAD_B);
        let pressed_combo = combo.is_pressed(&self.gamepad_state);

        let released_a = !pressed_a_after && pressed_a_before;
        let released_b = !pressed_b_after && pressed_b_before;

        if pressed_combo {
            BLOCK_XINPUT.store(true, Ordering::SeqCst);
            self.radial_menu_open_time = Instant::now();
        }

        let debounce_elapsed = self.radial_menu_open_time.elapsed() > RADIAL_MENU_DEBOUNCE;

        if BLOCK_XINPUT.load(Ordering::SeqCst) {
            let menu = self
                .radial_menu
                .iter()
                .map(|RadialMenu { label, .. }| label.as_str())
                .collect::<Vec<_>>();
            let x = self.gamepad_state.Gamepad.sThumbLX as f32;
            let y = -(self.gamepad_state.Gamepad.sThumbLY as f32);

            let norm = (x * x + y * y).sqrt();

            if norm > 10000.0 && debounce_elapsed {
                let scale = 1. / norm;
                let x = x * scale;
                let y = y * scale;
                self.gamepad_stick = ImVec2 { x, y };
            }

            let menu_out = radial_menu(ui, &menu, self.gamepad_stick, h * 0.1, h * 0.25);

            if released_a {
                if let Some(i) = menu_out {
                    self.radial_menu[i].key.keys(&mut self.press_queue);
                }
                BLOCK_XINPUT.store(false, Ordering::SeqCst);
            } else if released_b {
                BLOCK_XINPUT.store(false, Ordering::SeqCst);
            }
        }
    }

    fn render_logs(&mut self, ui: &imgui::Ui) {
        let io = ui.io();

        let [dw, dh] = io.display_size;
        let [ww, wh] = [dw * 0.3, 14.0 * 6.];

        let stack_tokens = vec![
            ui.push_style_var(StyleVar::WindowRounding(0.)),
            ui.push_style_var(StyleVar::FrameBorderSize(0.)),
            ui.push_style_var(StyleVar::WindowBorderSize(0.)),
        ];

        ui.window("##logs")
            .position_pivot([1., 1.])
            .position([dw * 0.95, dh * 0.8], Condition::Always)
            .flags({
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::ALWAYS_AUTO_RESIZE
                    | WindowFlags::NO_INPUTS
            })
            .size([ww, wh], Condition::Always)
            .bg_alpha(0.0)
            .build(|| {
                for _ in 0..5 {
                    ui.text("");
                }
                for l in self.log.iter().rev().take(3).rev() {
                    ui.text(&l.1);
                }
                ui.set_scroll_here_y();
            });

        for st in stack_tokens.into_iter().rev() {
            st.pop();
        }
    }

    fn set_font<'a>(&mut self, ui: &'a imgui::Ui) -> imgui::FontStackToken<'a> {
        let width = ui.io().display_size[0];
        let font_id = self
            .fonts
            .as_mut()
            .map(|fonts| {
                if width > 2000. {
                    fonts.big
                } else if width > 1200. {
                    fonts.normal
                } else {
                    fonts.small
                }
            })
            .unwrap();

        ui.push_font(font_id)
    }
}

impl ImguiRenderLoop for PracticeTool {
    fn before_render(&mut self, ctx: &mut Context, _: &mut dyn RenderContext) {
        self.release_queue.drain(..).for_each(|key| {
            ctx.io_mut().add_key_event(key, false);
        });
        self.press_queue.drain(..).for_each(|key| {
            ctx.io_mut().add_key_event(key, true);
            self.release_queue.push(key);
        });
    }

    fn render(&mut self, ui: &mut imgui::Ui) {
        let font_token = self.set_font(ui);

        let display = self.settings.display.is_pressed(ui);
        let hide = self.settings.hide.map(|k| k.is_pressed(ui)).unwrap_or(false);

        self.framecount += 1;

        if !ui.io().want_capture_keyboard && (display || hide) {
            self.ui_state = match (&self.ui_state, hide) {
                (UiState::Hidden, _) => UiState::Closed,
                (_, true) => UiState::Hidden,
                (UiState::MenuOpen, _) => UiState::Closed,
                (UiState::Closed, _) => UiState::MenuOpen,
            };

            match &self.ui_state {
                UiState::MenuOpen => {},
                UiState::Closed => self.pointers.cursor_show.set(false),
                UiState::Hidden => self.pointers.cursor_show.set(false),
            }
        }

        self.render_radial(ui);

        match &self.ui_state {
            UiState::MenuOpen => {
                self.pointers.cursor_show.set(true);
                self.render_visible(ui);
            },
            UiState::Closed => {
                self.render_closed(ui);
            },
            UiState::Hidden => {
                self.render_hidden(ui);
            },
        }

        for w in &mut self.widgets {
            w.log(self.log_tx.clone());
        }

        let now = Instant::now();
        self.log.extend(self.log_rx.try_iter().inspect(|log| info!("{}", log)).map(|l| (now, l)));
        self.log.retain(|(tm, _)| tm.elapsed() < std::time::Duration::from_secs(5));

        self.render_logs(ui);
        drop(font_token);
    }

    fn initialize(&mut self, ctx: &mut Context, _: &mut dyn RenderContext) {
        let fonts = ctx.fonts();
        self.fonts = Some(FontIDs {
            small: fonts.add_font(&[FontSource::TtfData {
                data: include_bytes!("../../lib/data/ComicMono.ttf"),
                size_pixels: 11.,
                config: None,
            }]),
            normal: fonts.add_font(&[FontSource::TtfData {
                data: include_bytes!("../../lib/data/ComicMono.ttf"),
                size_pixels: 18.,
                config: None,
            }]),
            big: fonts.add_font(&[FontSource::TtfData {
                data: include_bytes!("../../lib/data/ComicMono.ttf"),
                size_pixels: 24.,
                config: None,
            }]),
        });
    }
}

// Display some imgui debug information. Very expensive.
fn imgui_debug(ui: &Ui) {
    let io = ui.io();
    ui.text(format!("Mouse position     {:?}", io.mouse_pos));
    ui.text(format!("Mouse down         {:?}", io.mouse_down));
    ui.text(format!("Want capture mouse {:?}", io.want_capture_mouse));
    ui.text(format!("Want capture kbd   {:?}", io.want_capture_keyboard));
    ui.text(format!("Want text input    {:?}", io.want_text_input));
    ui.text(format!("Want set mouse pos {:?}", io.want_set_mouse_pos));
    ui.text(format!("Any item active    {:?}", ui.is_any_item_active()));
    ui.text(format!("Any item hovered   {:?}", ui.is_any_item_hovered()));
    ui.text(format!("Any item focused   {:?}", ui.is_any_item_focused()));
    ui.text(format!("Any mouse down     {:?}", ui.is_any_mouse_down()));
}
