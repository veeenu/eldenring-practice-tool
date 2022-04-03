#![feature(once_cell)]

use std::path::Path;

use imgui::*;

use hudhook::hooks::dx12::{ImguiRenderLoop, ImguiRenderLoopFlags};

struct PracticeTool {
    done1: bool,
    done2: bool,
}

impl PracticeTool {
    fn new() -> Self {
        use simplelog::*;

        hudhook::utils::alloc_console();
        log_panics::init();

        CombinedLogger::init(vec![
            TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed),
            WriteLogger::new(
                LevelFilter::Trace,
                Config::default(),
                std::fs::File::create(Path::new("eldenring-practice-tool.log")).unwrap(),
            ),
        ])
        .ok();

        PracticeTool {
            done1: false,
            done2: false,
        }
    }
}

impl ImguiRenderLoop for PracticeTool {
    fn render(&mut self, ui: &mut imgui::Ui, flags: &ImguiRenderLoopFlags) {
        Window::new("##window").size([320., 70.], Condition::Always).build(ui, || {
            ui.text("Press \"P\" to pay respects");
            if ui.is_key_index_released('P' as i32) {
                self.done1 = true;
            }
            if self.done1 {
                ui.text("Haha maidenless peepoArriveDabThenLeave");
                if !self.done2 {
                    unsafe {
                        // let addr = (0x7FF628710000 as u64 + 0x3c6a700 as u64) as *mut usize;
                        let addr = (0x7ff7b1b80000 as u64 + 0x3c6a700 as u64) as *mut usize;
                        info!("{:p}", addr);
                        let addr = ((*addr) + 0x8) as *mut usize;
                        info!("{:p}", addr);
                        let addr = ((*addr) + 0x5d) as *mut u8;
                        info!("{:p}", addr);
                        *addr = 1;
                    }
                }
                self.done2 = true;
            }
        });
    }
}

hudhook::hudhook!(|| { hudhook::hooks::dx12::hook_imgui(PracticeTool::new()) });
