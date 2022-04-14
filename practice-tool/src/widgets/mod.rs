use std::sync::Arc;

use parking_lot::Mutex;

pub(crate) const BUTTON_WIDTH: f32 = 200.;
pub(crate) const BUTTON_HEIGHT: f32 = 0.;

pub(crate) mod cycle_speed;
pub(crate) mod flag;
pub(crate) mod multiflag;
// pub(crate) mod item_spawn;
pub(crate) mod deathcam;
pub(crate) mod position;
pub(crate) mod quitout;
pub(crate) mod savefile_manager;
pub(crate) mod runes;

pub(crate) trait Widget: Send + Sync + std::fmt::Debug {
    fn render(&mut self, ui: &imgui::Ui);
    fn interact(&mut self) {}
    fn interact_ui(&mut self) {}

    fn enter(&self, _ui: &imgui::Ui) -> Option<Arc<Mutex<Box<dyn Widget>>>> {
        None
    }
    fn exit(&self, _ui: &imgui::Ui) {}
    fn cursor_down(&mut self) {}
    fn cursor_up(&mut self) {}

    fn want_enter(&mut self) -> bool {
        false
    }
    fn want_exit(&mut self) -> bool {
        false
    }
    fn log(&mut self) -> Option<Vec<String>> {
        None
    }
}
