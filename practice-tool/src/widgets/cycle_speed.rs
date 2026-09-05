use std::cmp::Ordering;
use std::fmt::Write;

use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::store_value::{ReadWrite, StoreValue};
use practice_tool_core::widgets::Widget;

#[derive(Debug)]
struct CycleSpeed {
    ptr: [SpeedTarget; 2],
    values: Vec<f32>,
    current: Option<f32>,
    desired: Option<f32>,
    label: String,
}

impl CycleSpeed {
    fn new(values: &[f32], ptr: [SpeedTarget; 2]) -> Self {
        let mut values = values.to_vec();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        CycleSpeed { ptr, values, current: None, desired: None, label: String::new() }
    }
}

impl ReadWrite for CycleSpeed {
    fn read(&mut self) -> bool {
        self.current = self.ptr[0].read();

        // Re-apply the value after the game recreates the animation state.
        if let Some(desired) = self.desired {
            if self.current != Some(desired) {
                self.ptr[0].write(desired);
                self.current = self.ptr[0].read();
            }
            if let SpeedTargetResolution::Ready(torrent) = self.ptr[1].resolve() {
                if torrent.read() != Some(desired) {
                    torrent.write(desired);
                }
            }
        }

        self.label.clear();

        match self.current {
            Some(c) => write!(self.label, "Speed [{c:.1}x]").ok(),
            None => write!(self.label, "Speed").ok(),
        };

        self.current.is_some()
    }

    fn write(&mut self) {
        let next = *self
            .current
            .and_then(|current| self.values.iter().find(|&&x| x > current))
            .unwrap_or_else(|| self.values.first().unwrap_or(&1.0));

        let player = match self.ptr[0].resolve() {
            SpeedTargetResolution::Ready(pointer) => pointer,
            SpeedTargetResolution::Unavailable(_) => return,
        };
        let torrent = match self.ptr[1].resolve() {
            SpeedTargetResolution::Ready(pointer) => Some(pointer),
            SpeedTargetResolution::Unavailable(_) => return,
        };

        let Some(previous_player) = player.read() else {
            return;
        };
        let previous_torrent = match torrent.as_ref() {
            Some(pointer) => match pointer.read() {
                Some(value) => Some(value),
                None => return,
            },
            None => None,
        };

        if player.write(next).is_none() || player.read() != Some(next) {
            player.write(previous_player);
            return;
        }

        let Some(torrent) = torrent else {
            self.desired = Some(next);
            return;
        };
        let torrent_write_ok = torrent.write(next).is_some();
        let torrent_after_write = torrent.read();
        if !torrent_write_ok || torrent_after_write != Some(next) {
            player.write(previous_player);
            if let Some(previous_torrent) = previous_torrent {
                torrent.write(previous_torrent);
            }
            return;
        }
        self.desired = Some(next);
    }

    fn label(&self) -> &str {
        &self.label
    }
}

pub(crate) fn cycle_speed(
    values: &[f32],
    ptr: [SpeedTarget; 2],
    key: Option<Key>,
) -> Box<dyn Widget> {
    Box::new(StoreValue::new(CycleSpeed::new(values, ptr), key))
}
