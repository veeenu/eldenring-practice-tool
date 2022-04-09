use libeldenring::prelude::*;

use crate::util::KeyState;

use super::Widget;

#[derive(Debug)]
pub(crate) struct SavePosition {
    pos: Position,
    hotkey: KeyState,
    modifier: KeyState,
    saved_position: [f32; 4],
}

impl SavePosition {
    pub(crate) fn new(pos: Position, hotkey: KeyState, modifier: KeyState) -> Self {
        SavePosition {
            pos,
            hotkey,
            modifier,
            saved_position: [0f32; 4],
        }
    }

    fn save_position(&mut self) {
        if let (Some(x), Some(y), Some(z), Some(angle)) = (
            self.pos.x.read(),
            self.pos.y.read(),
            self.pos.z.read(),
            self.pos.angle.read(),
        ) {
            self.saved_position = [x, y, z, angle];
        }
    }

    fn load_position(&mut self) {
        let [x, y, z, angle] = self.saved_position;
        self.pos.x.write(x);
        self.pos.y.write(y);
        self.pos.z.write(z);
        self.pos.angle.write(angle);
    }
}

impl Widget for SavePosition {
    fn render(&mut self, ui: &imgui::Ui) {
        let (x, y, z, angle) = (
            self.pos.x.read(),
            self.pos.y.read(),
            self.pos.z.read(),
            self.pos.angle.read(),
        );
        let saved_pos = self.saved_position;

        let (read_pos, valid) = if let (Some(x), Some(y), Some(z), Some(angle)) = (x, y, z, angle) {
            ([x, y, z, angle], true)
        } else {
            ([0f32; 4], false)
        };

        let _token = ui.begin_disabled(!valid);

        if ui.button_with_size(
            format!("Load ({})", self.hotkey),
            [super::BUTTON_WIDTH * 0.33 - 4., super::BUTTON_HEIGHT],
        ) {
            self.load_position();
        }
        ui.same_line();
        if ui.button_with_size(
            format!("Save ({} + {})", self.modifier, self.hotkey),
            [super::BUTTON_WIDTH * 0.67 - 4., super::BUTTON_HEIGHT],
        ) {
            self.save_position();
        }
        ui.text(format!(
            "[{:6.2} {:6.2} {:6.2} {:6.2}]",
            read_pos[0], read_pos[1], read_pos[2], read_pos[3]
        ));
        ui.text(format!(
            "[{:6.2} {:6.2} {:6.2} {:6.2}]",
            saved_pos[0], saved_pos[1], saved_pos[2], saved_pos[3],
        ));
    }

    fn interact(&mut self) {
        let key_up = self.hotkey.keyup();
        let mod_down = self.modifier.is_key_down();

        if key_up && mod_down {
            self.save_position();
        } else if key_up {
            self.load_position();
        }
    }
}
