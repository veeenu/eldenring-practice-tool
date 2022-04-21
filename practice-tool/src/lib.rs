#![feature(once_cell)]

mod config;
mod util;
mod widgets;

use std::time::Instant;

use imgui::*;

use hudhook::hooks::dx12::{ImguiRenderLoop, ImguiRenderLoopFlags};
use libeldenring::prelude::*;

use crate::widgets::Widget;

struct PracticeTool {
    pointers: Pointers,
    widgets: Vec<Box<dyn Widget>>,
    config: config::Config,
    log: Vec<(Instant, String)>,
    is_shown: bool,
}

impl PracticeTool {
    fn new() -> Self {
        use simplelog::*;

        hudhook::utils::alloc_console();
        log_panics::init();

        fn load_config() -> Result<config::Config, String> {
            let config_path = crate::util::get_dll_path()
                .map(|mut path| {
                    path.pop();
                    path.push("jdsd_er_practice_tool.toml");
                    path
                })
                .ok_or_else(|| "Couldn't find config file".to_string())?;
            let config_content = std::fs::read_to_string(config_path)
                .map_err(|e| format!("Couldn't read config file: {}", e))?;
            println!("{}", config_content);
            config::Config::parse(&config_content).map_err(String::from)
        }

        let (config, config_err) = match load_config() {
            Ok(config) => (config, None),
            Err(e) => (config::Config::default(), Some(e)),
        };

        let log_file = crate::util::get_dll_path()
            .map(|mut path| {
                path.pop();
                path.push("jdsd_er_practice_tool.log");
                path
            })
            .map(std::fs::File::create);

        match log_file {
            Some(Ok(log_file)) => {
                CombinedLogger::init(vec![
                    TermLogger::new(
                        config.settings.log_level.inner(),
                        Config::default(),
                        TerminalMode::Mixed,
                        ColorChoice::Auto,
                    ),
                    WriteLogger::new(
                        config.settings.log_level.inner(),
                        Config::default(),
                        log_file,
                    ),
                ])
                .ok();
            }
            e => {
                CombinedLogger::init(vec![TermLogger::new(
                    LevelFilter::Debug, // config.settings.log_level.to_level_filter(),
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                )])
                .ok();

                match e {
                    None => error!("Could not construct log file path"),
                    Some(Err(e)) => error!("Could not initialize log file: {:?}", e),
                    _ => unreachable!(),
                }
            }
        }

        if let Some(err) = config_err {
            debug!("{}", err);
        }

        let pointers = Pointers::new();
        debug!("{:#?}", pointers);
        let widgets = config.make_commands(&pointers);

        PracticeTool {
            pointers,
            widgets,
            config,
            is_shown: false,
            log: Default::default(),
        }
    }

    fn render_visible(&mut self, ui: &mut imgui::Ui, flags: &ImguiRenderLoopFlags) {
        imgui::Window::new("##tool_window")
            .position([16., 16.], Condition::Always)
            .bg_alpha(0.8)
            .flags({
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::ALWAYS_AUTO_RESIZE
            })
            .build(ui, || {
                for w in self.widgets.iter_mut() {
                    w.render(ui);
                }
                if flags.focused && !ui.io().want_capture_keyboard {
                    for w in self.widgets.iter_mut() {
                        w.interact();
                    }
                }

                if ui.button_with_size("Close", [widgets::BUTTON_WIDTH, widgets::BUTTON_HEIGHT]) {
                    self.is_shown = false;
                    self.pointers.cursor_show.set(false);
                }
            });
    }

    fn render_closed(&mut self, ui: &mut imgui::Ui, flags: &ImguiRenderLoopFlags) {
        let stack_tokens = vec![
            ui.push_style_var(StyleVar::WindowRounding(0.)),
            ui.push_style_var(StyleVar::FrameBorderSize(0.)),
            ui.push_style_var(StyleVar::WindowBorderSize(0.)),
        ];
        imgui::Window::new("##msg_window")
            .position([16., 16.], Condition::Always)
            .bg_alpha(0.0)
            .flags({
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::ALWAYS_AUTO_RESIZE
            })
            .build(ui, || {
                ui.text("johndisandonato's Elden Ring Practice Tool");

                ui.same_line();

                if ui.small_button("Open") {
                    self.is_shown = true;
                }

                ui.same_line();

                if ui.small_button("Help") {
                    ui.open_popup("##help_window");
                }

                PopupModal::new("##help_window")
                    .resizable(false)
                    .movable(false)
                    .title_bar(false)
                    .build(ui, || {
                        ui.text("Elden Ring Practice Tool");
                        ui.separator();
                        ui.text(format!(
                            "Press the {} key to open/close the tool's\n\
                             interface.\n\n\
                             You can toggle flags/launch commands by\n\
                             clicking in the UI or by pressing\n\
                             the hotkeys (in the parentheses).\n\n\
                             You can configure your tool by editing\n\
                             the jdsd_er_practice_tool.toml file with\n\
                             a text editor. If you break something,\n\
                             just download a fresh file!\n\n\
                             Thank you for using my tool! <3\n",
                            self.config.settings.display
                        ));
                        ui.separator();
                        ui.text("-- johndisandonato");
                        ui.text("   https://twitch.tv/johndisandonato");
                        if ui.is_item_clicked() {
                            open::that("https://twitch.tv/johndisandonato").ok();
                        }
                        ui.separator();
                        if ui.button("Close") {
                            ui.close_current_popup();
                        }
                        ui.same_line();
                        if ui.button("Submit issue") {
                            open::that(
                                "https://github.com/veeenu/eldenring-practice-tool/issues/new",
                            )
                            .ok();
                        }
                    });

                if let Some(igt) = self.pointers.igt.read() {
                    let millis = (igt % 1000) / 10;
                    let total_seconds = igt / 1000;
                    let seconds = total_seconds % 60;
                    let minutes = total_seconds / 60 % 60;
                    let hours = total_seconds / 3600;
                    ui.text(format!(
                        "IGT {:02}:{:02}:{:02}.{:02}",
                        hours, minutes, seconds, millis
                    ));
                }

                if flags.focused && !ui.io().want_capture_keyboard {
                    for w in self.widgets.iter_mut() {
                        w.interact();
                    }
                }
            });

        for st in stack_tokens.into_iter().rev() {
            st.pop();
        }
    }

    fn render_logs(&mut self, ui: &mut imgui::Ui, _flags: &ImguiRenderLoopFlags) {
        let io = ui.io();

        let [dw, dh] = io.display_size;
        let [ww, wh] = [dw * 0.3, 14.0 * 6.];

        let stack_tokens = vec![
            ui.push_style_var(StyleVar::WindowRounding(0.)),
            ui.push_style_var(StyleVar::FrameBorderSize(0.)),
            ui.push_style_var(StyleVar::WindowBorderSize(0.)),
        ];

        Window::new("##logs")
            .position_pivot([1., 1.])
            .position([dw * 0.95, dh * 0.8], Condition::Always)
            .flags({
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::ALWAYS_AUTO_RESIZE
            })
            .size([ww, wh], Condition::Always)
            .bg_alpha(0.0)
            .build(ui, || {
                for _ in 0..20 {
                    ui.text("");
                }
                for l in self.log.iter() {
                    ui.text(&l.1);
                }
                ui.set_scroll_here_y();
            });

        for st in stack_tokens.into_iter().rev() {
            st.pop();
        }
    }
}

impl ImguiRenderLoop for PracticeTool {
    fn render(&mut self, ui: &mut imgui::Ui, flags: &ImguiRenderLoopFlags) {
        if flags.focused
            && !ui.io().want_capture_keyboard
            && self.config.settings.display.keyup()
        {
            self.is_shown = !self.is_shown;
            if !self.is_shown {
                self.pointers.cursor_show.set(false);
            }
        }

        if self.is_shown {
            self.pointers.cursor_show.set(true);
            self.render_visible(ui, flags);
        } else {
            self.render_closed(ui, flags);
        }

        for w in &mut self.widgets {
            if let Some(logs) = w.log() {
                let now = Instant::now();
                self.log.extend(logs.into_iter().map(|l| (now, l)));
            }
            self.log
                .retain(|(tm, _)| tm.elapsed() < std::time::Duration::from_secs(5));
        }

        self.render_logs(ui, flags);
    }
}

hudhook::hudhook!(|| { hudhook::hooks::dx12::hook_imgui(PracticeTool::new()) });
