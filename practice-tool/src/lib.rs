// johndisandonato's Elden Ring Practice Tool
// Copyright (C) 2022  johndisandonato <https://github.com/veeenu>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![feature(once_cell)]

mod config;
mod util;
mod widgets;

use std::time::Instant;

use const_format::formatcp;
use imgui::*;
use pkg_version::*;

use hudhook::hooks::dx12::{ImguiRenderLoop, ImguiRenderLoopFlags};
use libeldenring::prelude::*;

use crate::widgets::Widget;

struct FontIDs {
    small: FontId,
    normal: FontId,
    big: FontId,
}

unsafe impl Send for FontIDs {}
unsafe impl Sync for FontIDs {}

struct PracticeTool {
    pointers: Pointers,
    widgets: Vec<Box<dyn Widget>>,
    config: config::Config,
    log: Vec<(Instant, String)>,
    is_shown: bool,
    fonts: Option<FontIDs>,
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

        let log_level = config.settings.log_level.inner();

        if log_level < LevelFilter::Debug {
            hudhook::utils::free_console();
        }

        match log_file {
            Some(Ok(log_file)) => {
                CombinedLogger::init(vec![
                    TermLogger::new(
                        log_level,
                        Config::default(),
                        TerminalMode::Mixed,
                        ColorChoice::Auto,
                    ),
                    WriteLogger::new(log_level, Config::default(), log_file),
                ])
                .ok();
            }
            e => match e {
                None => error!("Could not construct log file path"),
                Some(Err(e)) => error!("Could not initialize log file: {:?}", e),
                _ => unreachable!(),
            },
        }

        if let Some(err) = config_err {
            error!("{}", err);
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
                if let Some(mut spectral_steed_whistle) =
                    epg.find(|i| i.id == 130).and_then(|p| p.param)
                {
                    spectral_steed_whistle.icon_id = 12;
                };
            },
        );

        let pointers = Pointers::new();
        let widgets = config.make_commands(&pointers);
        info!("Practice tool initialized");

        PracticeTool {
            pointers,
            widgets,
            config,
            is_shown: false,
            log: Default::default(),
            fonts: None,
        }
    }

    fn render_visible(&mut self, ui: &imgui::Ui, flags: &ImguiRenderLoopFlags) {
        let [dw, dh] = { ui.io().display_size };
        imgui::Window::new("##tool_window")
            .position([16., 16.], Condition::Always)
            .size_constraints([240., 0.], [dw - 70., dh - 70.])
            .bg_alpha(0.8)
            .flags({
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    // | WindowFlags::NO_SCROLLBAR
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

                if ui.button_with_size(
                    "Close",
                    [
                        widgets::BUTTON_WIDTH * widgets::scaling_factor(ui),
                        widgets::BUTTON_HEIGHT,
                    ],
                ) {
                    self.is_shown = false;
                    self.pointers.cursor_show.set(false);
                }
            });
    }

    fn render_closed(&mut self, ui: &imgui::Ui, flags: &ImguiRenderLoopFlags) {
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
                        ui.text(formatcp!(
                            "Elden Ring Practice Tool v{}.{}.{}",
                            pkg_version_major!() as usize,
                            pkg_version_minor!() as usize,
                            pkg_version_patch!() as usize,
                        ));
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

    fn render_logs(&mut self, ui: &imgui::Ui, _flags: &ImguiRenderLoopFlags) {
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
    fn render(&mut self, ui: &mut imgui::Ui, flags: &ImguiRenderLoopFlags) {
        let font_token = self.set_font(ui);

        if flags.focused && !ui.io().want_capture_keyboard && self.config.settings.display.keyup() {
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
        drop(font_token);
    }

    fn initialize(&mut self, ctx: &mut imgui::Context) {
        let mut fonts = ctx.fonts();
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

hudhook::hudhook!(|| { hudhook::hooks::dx12::hook_imgui(PracticeTool::new()) });
