use std::fmt::Write;

use libeldenring::prelude::Position as ErPosition;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::nudge_position::NudgePositionStorage;
use practice_tool_core::widgets::position::{Position, PositionStorage};
use practice_tool_core::widgets::Widget;

pub(super) struct SavePosition {
    global_position: ErPosition,
    chunk_position: ErPosition,
    torrent_chunk_position: ErPosition,

    label_current: String,
    label_stored: String,
    valid: bool,
    nudge: f32,

    saved_position: [f32; 5],
    saved_map_id: u32,
}

impl SavePosition {
    pub(super) fn new(
        global_position: ErPosition,
        chunk_position: ErPosition,
        torrent_chunk_position: ErPosition,
        nudge: f32,
    ) -> Self {
        Self {
            global_position,
            chunk_position,
            torrent_chunk_position,
            saved_position: [0.0; 5],
            saved_map_id: 0u32,
            label_current: String::new(),
            label_stored: String::new(),
            valid: false,
            nudge,
        }
    }
}

impl PositionStorage for SavePosition {
    fn save(&mut self) {
        if let (Some([x, y, z, _, _]), Some([_, _, _, r1, r2]), Some(m)) = (
            self.global_position.read(),
            self.chunk_position.read(),
            self.global_position.read_map_id(),
        ) {
            self.saved_position = [x, y, z, r1, r2];
            self.saved_map_id = m;
            self.valid = true;
        } else {
            self.valid = false;
        }
    }

    fn load(&mut self) {
        if let (Some([gx, gy, gz, _, _]), Some([cx, cy, cz, _, _])) =
            (self.global_position.read(), self.chunk_position.read())
        {
            let [sx, sy, sz, sr1, sr2] = self.saved_position;
            self.chunk_position.write([sx - gx + cx, sy - gy + cy, sz - gz + cz, sr1, sr2]);
            self.chunk_position.write_map_id(self.saved_map_id);

            if let Some([tcx, tcy, tcz, _, _]) = self.torrent_chunk_position.read() {
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

    fn display_current(&mut self) -> &str {
        self.label_current.clear();

        let (read_pos, valid) = if let (Some([x, y, z, _, _]), Some(angle)) =
            (self.global_position.read(), self.chunk_position.angle1.read())
        {
            ([x, y, z, angle], true)
        } else {
            ([0f32; 4], false)
        };

        self.valid = valid;

        write!(
            self.label_current,
            "{:7.1} {:7.1} {:7.1} {:7.1}",
            read_pos[0], read_pos[1], read_pos[2], read_pos[3]
        )
        .ok();

        &self.label_current
    }

    fn display_stored(&mut self) -> &str {
        self.label_stored.clear();

        let [x, y, z, a, _] = self.saved_position;

        write!(self.label_stored, "{:7.1} {:7.1} {:7.1} {:7.1}", x, y, z, a).ok();

        &self.label_stored
    }

    fn is_valid(&self) -> bool {
        self.valid
    }
}

impl NudgePositionStorage for SavePosition {
    fn nudge_up(&mut self) {
        if let Some(y) = self.chunk_position.y.read() {
            self.chunk_position.y.write(y + self.nudge);
        }
        if let Some(y) = self.torrent_chunk_position.y.read() {
            self.torrent_chunk_position.y.write(y + self.nudge);
        }
    }

    fn nudge_down(&mut self) {
        if let Some(y) = self.chunk_position.y.read() {
            self.chunk_position.y.write(y - self.nudge);
        }
        if let Some(y) = self.torrent_chunk_position.y.read() {
            self.torrent_chunk_position.y.write(y - self.nudge);
        }
    }
}

pub(crate) fn save_position(
    global_position: ErPosition,
    chunk_position: ErPosition,
    torrent_chunk_position: ErPosition,
    key_load: Option<Key>,
    key_save: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(Position::new(
        SavePosition::new(global_position, chunk_position, torrent_chunk_position, 0.0),
        key_load,
        key_save,
    ))
}

// use super::Widget;
// use crate::util::KeyState;
//
// #[derive(Debug)]
// pub(crate) struct SavePosition {
//     global_position: Position,
//     chunk_position: Position,
//     torrent_chunk_position: Position,
//     hotkey: KeyState,
//     modifier: KeyState,
//     saved_position: [f32; 5],
//     saved_map_id: u32,
// }
//
// impl SavePosition {
//     pub(crate) fn new(
//         global_position: Position,
//         chunk_position: Position,
//         torrent_chunk_position: Position,
//         hotkey: KeyState,
//         modifier: KeyState,
//     ) -> Self {
//         SavePosition {
//             global_position,
//             chunk_position,
//             torrent_chunk_position,
//             hotkey,
//             modifier,
//             saved_position: [0f32; 5],
//             saved_map_id: 0u32,
//         }
//     }
//
//     fn save_position(&mut self) {
//         if let (Some([x, y, z, _, _]), Some([_, _, _, r1, r2]), Some(m)) = (
//             self.global_position.read(),
//             self.chunk_position.read(),
//             self.global_position.read_map_id(),
//         ) {
//             self.saved_position = [x, y, z, r1, r2];
//             self.saved_map_id = m;
//         }
//     }
//
//     fn load_position(&mut self) {
//         if let (Some([gx, gy, gz, _, _]), Some([cx, cy, cz, _, _])) =
//             (self.global_position.read(), self.chunk_position.read())
//         {
//             let [sx, sy, sz, sr1, sr2] = self.saved_position;
//             self.chunk_position.write([sx - gx + cx, sy - gy + cy, sz - gz +
// cz, sr1, sr2]);
// self.chunk_position.write_map_id(self.saved_map_id);
//
//             if let Some([tcx, tcy, tcz, _, _]) =
// self.torrent_chunk_position.read() {
// self.torrent_chunk_position.write([                     sx - gx + tcx,
//                     sy - gy + tcy,
//                     sz - gz + tcz,
//                     sr1,
//                     sr2,
//                 ]);
//             }
//         }
//     }
// }
//
// impl Widget for SavePosition {
//     fn render(&mut self, ui: &imgui::Ui) {
//         let saved_pos = self.saved_position;
//
//         let (read_pos, valid) = if let (Some([x, y, z, _, _]), Some(angle)) =
//             (self.global_position.read(), self.chunk_position.angle1.read())
//         {
//             ([x, y, z, angle], true)
//         } else {
//             ([0f32; 4], false)
//         };
//
//         let _token = ui.begin_disabled(!valid);
//         let button_width = super::BUTTON_WIDTH * super::scaling_factor(ui);
//
//         if ui.button_with_size(format!("Load ({})", self.hotkey), [
//             button_width * 0.33 - 4.,
//             super::BUTTON_HEIGHT,
//         ]) {
//             self.load_position();
//         }
//         ui.same_line();
//         if ui.button_with_size(format!("Save ({} + {})", self.modifier,
// self.hotkey), [             button_width * 0.67 - 4.,
//             super::BUTTON_HEIGHT,
//         ]) {
//             self.save_position();
//         }
//         ui.text(format!(
//             "{:7.1} {:7.1} {:7.1} {:7.1}",
//             read_pos[0], read_pos[1], read_pos[2], read_pos[3]
//         ));
//         ui.text(format!(
//             "{:7.1} {:7.1} {:7.1} {:7.1}",
//             saved_pos[0], saved_pos[1], saved_pos[2], saved_pos[3],
//         ));
//     }
//
//     fn interact(&mut self, ui: &imgui::Ui) {
//         if ui.is_any_item_active() {
//             return;
//         }
//
//         let key_up = self.hotkey.keyup(ui);
//         let mod_down = self.modifier.is_key_down(ui);
//
//         if key_up && mod_down {
//             self.save_position();
//         } else if key_up {
//             self.load_position();
//         }
//     }
// }
