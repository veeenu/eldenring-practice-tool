use libeldenring::prelude::*;

use crate::util::KeyState;

use super::Widget;

#[derive(Debug)]
pub(crate) struct SavePosition {
    global_position: Position,
    chunk_position: Position,
    hotkey: KeyState,
    modifier: KeyState,
    saved_position: [f32; 4],
}

impl SavePosition {
    pub(crate) fn new(global_position: Position, chunk_position: Position, hotkey: KeyState, modifier: KeyState) -> Self {
        SavePosition {
            global_position,
            chunk_position,
            hotkey,
            modifier,
            saved_position: [0f32; 4],
        }
    }

    fn save_position(&mut self) {
        if let (Some(x), Some(y), Some(z), Some(angle)) = (
            self.global_position.x.read(),
            self.global_position.y.read(),
            self.global_position.z.read(),
            self.global_position.angle.read(),
        ) {
            self.saved_position = [x, y, z, angle];
        }
    }

    fn load_position(&mut self) {
        let [sx, sy, sz, sr] = self.saved_position;
        if let (Some(gx), Some(gy), Some(gz), Some(gr),
            Some(cx), Some(cy), Some(cz), Some(cr),

            ) = (
            self.global_position.x.read(),
            self.global_position.y.read(),
            self.global_position.z.read(),
            self.global_position.angle.read(),
            self.chunk_position.x.read(),
            self.chunk_position.y.read(),
            self.chunk_position.z.read(),
            self.chunk_position.angle.read(),
        ) {
            self.chunk_position.x.write(sx - gx + cx);
            self.chunk_position.y.write(sy - gy + cy);
            self.chunk_position.z.write(sz - gz + cz);
            self.chunk_position.angle.write(sr - gr + cr);
        }
    }
}

impl Widget for SavePosition {
    fn render(&mut self, ui: &imgui::Ui) {
        let (x, y, z, angle) = (
            self.global_position.x.read(),
            self.global_position.y.read(),
            self.global_position.z.read(),
            self.global_position.angle.read(),
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
            "{:7.1} {:7.1} {:7.1} {:7.1}",
            read_pos[0], read_pos[1], read_pos[2], read_pos[3]
        ));
        ui.text(format!(
            "{:7.1} {:7.1} {:7.1} {:7.1}",
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
