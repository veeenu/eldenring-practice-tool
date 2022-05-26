use libeldenring::prelude::*;

use crate::util::KeyState;

use super::Widget;

#[derive(Debug)]
pub(crate) struct NudgePosition {
    chunk_position: Position,
    nudge: f32,
    nudge_up: Option<KeyState>,
    nudge_down: Option<KeyState>,
    nudge_up_label: String,
    nudge_down_label: String,
}

impl NudgePosition {
    pub(crate) fn new(
        chunk_position: Position,
        nudge: f32,
        nudge_up: Option<KeyState>,
        nudge_down: Option<KeyState>,
    ) -> Self {
        let nudge_up_label = if let Some(k) = &nudge_up {
            format!("Nudge up ({})", k)
        } else {
            "Nudge up".to_string()
        };
        let nudge_down_label = if let Some(k) = &nudge_down {
            format!("Nudge down ({})", k)
        } else {
            "Nudge down".to_string()
        };
        NudgePosition {
            chunk_position,
            nudge,
            nudge_up,
            nudge_down,
            nudge_up_label,
            nudge_down_label,
        }
    }

    fn do_nudge_up(&mut self) {
        if let Some(y) = self.chunk_position.y.read() {
            self.chunk_position.y.write(y + self.nudge);
        }
    }

    fn do_nudge_down(&mut self) {
        if let Some(y) = self.chunk_position.y.read() {
            self.chunk_position.y.write(y - self.nudge);
        }
    }
}

impl Widget for NudgePosition {
    fn render(&mut self, ui: &imgui::Ui) {
        let valid = self.chunk_position.y.eval().is_some();
        let _token = ui.begin_disabled(!valid);

        let button_width = super::BUTTON_WIDTH * super::scaling_factor(ui);

        if ui.button_with_size(
            &self.nudge_up_label,
            [button_width * 0.5 - 4., super::BUTTON_HEIGHT],
        ) {
            self.do_nudge_up();
        }
        ui.same_line();
        if ui.button_with_size(
            &self.nudge_down_label,
            [button_width * 0.5 - 4., super::BUTTON_HEIGHT],
        ) {
            self.do_nudge_down();
        }
    }

    fn interact(&mut self) {
        if let Some(true) = self.nudge_up.as_ref().map(|c| c.is_key_down()) {
            self.do_nudge_up();
        } else if let Some(true) = self.nudge_down.as_ref().map(|c| c.is_key_down()) {
            self.do_nudge_down();
        }
    }
}
