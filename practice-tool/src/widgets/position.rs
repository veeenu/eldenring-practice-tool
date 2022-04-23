use libeldenring::prelude::*;

use crate::util::KeyState;

use super::Widget;

#[derive(Debug)]
pub(crate) struct SavePosition {
    global_position: Position,
    chunk_position: Position,
    torrent_chunk_position: Position,
    hotkey: KeyState,
    modifier: KeyState,
    saved_position: [f32; 5],
}

impl SavePosition {
    pub(crate) fn new(
        global_position: Position,
        chunk_position: Position,
        torrent_chunk_position: Position,
        hotkey: KeyState,
        modifier: KeyState,
    ) -> Self {
        SavePosition {
            global_position,
            chunk_position,
            torrent_chunk_position,
            hotkey,
            modifier,
            saved_position: [0f32; 5],
        }
    }

    fn save_position(&mut self) {
        if let (Some([x, y, z, _, _]), Some([_, _, _, r1, r2])) =
            (self.global_position.read(), self.chunk_position.read())
        {
            self.saved_position = [x, y, z, r1, r2];
        }
    }

    fn load_position(&mut self) {
        if let (Some([gx, gy, gz, _, _]), Some([cx, cy, cz, _, _]), Some([tcx, tcy, tcz, _, _])) = (
            self.global_position.read(),
            self.chunk_position.read(),
            self.torrent_chunk_position.read(),
        ) {
            let [sx, sy, sz, sr1, sr2] = self.saved_position;

            use std::f32::consts::PI;

            self.chunk_position
                .write([sx - gx + cx, sy - gy + cy, sz - gz + cz, sr1, sr2]);

            self.torrent_chunk_position.write([
                sx - gx + tcx,
                sy - gy + tcy,
                sz - gz + tcz,
                sr1,
                sr2,
            ]);
        }
    }
}

impl Widget for SavePosition {
    fn render(&mut self, ui: &imgui::Ui) {
        let saved_pos = self.saved_position;

        let (read_pos, valid) = if let (Some([x, y, z, _, _]), Some(angle)) = (
            self.global_position.read(),
            self.chunk_position.angle1.read(),
        ) {
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
