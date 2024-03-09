use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::nudge_position::NudgePosition;
use practice_tool_core::widgets::Widget;

use crate::widgets::position::SavePosition;

pub(crate) fn nudge_position(
    global_position: Position,
    chunk_position: Position,
    torrent_chunk_position: Position,
    nudge: f32,
    key_nudge_up: Option<Key>,
    key_nudge_down: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(NudgePosition::new(
        SavePosition::new(global_position, chunk_position, torrent_chunk_position, nudge),
        key_nudge_up,
        key_nudge_down,
    ))
}
