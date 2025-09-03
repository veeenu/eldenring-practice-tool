use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::stats_editor::{Datum, Stats, StatsEditor};
use practice_tool_core::widgets::Widget;

#[derive(Debug)]
struct CharacterStatsEdit {
    stats_ptr: PointerChain<CharacterStats>,
    points_ptr: PointerChain<CharacterPoints>,
    blessings_ptr: Option<PointerChain<CharacterBlessings>>,
    stats: Option<CharacterStats>,
    points: Option<CharacterPoints>,
    blessings: Option<CharacterBlessings>,
}

impl Stats for CharacterStatsEdit {
    fn data(&mut self) -> Option<impl Iterator<Item = Datum<'_>>> {
        self.stats.as_mut().map(|s| {
            let mut stats_data = vec![
                Datum::int("Level", &mut s.level, 1, 713),
                Datum::int("Vigor", &mut s.vigor, 1, 99),
                Datum::int("Mind", &mut s.mind, 1, 99),
                Datum::int("Endurance", &mut s.endurance, 1, 99),
                Datum::int("Strength", &mut s.strength, 1, 99),
                Datum::int("Dexterity", &mut s.dexterity, 1, 99),
                Datum::int("Intelligence", &mut s.intelligence, 1, 99),
                Datum::int("Faith", &mut s.faith, 1, 99),
                Datum::int("Arcane", &mut s.arcane, 1, 99),
                Datum::int("Souls", &mut s.runes, 0, i32::MAX),
            ];
            if let Some(p) = self.points.as_mut() {
                stats_data.insert(stats_data.len(), Datum::separator());
                stats_data.insert(stats_data.len(), Datum::int("HP", &mut p.hp, 0, i32::MAX));
                stats_data.insert(stats_data.len(), Datum::int("FP", &mut p.fp, 0, i32::MAX));
                stats_data.insert(stats_data.len(), Datum::int("SP", &mut p.stamina, 0, i32::MAX));
                stats_data
                    .insert(stats_data.len(), Datum::int("Max HP", &mut p.max_hp, 0, i32::MAX));
                stats_data
                    .insert(stats_data.len(), Datum::int("Max FP", &mut p.max_fp, 0, i32::MAX));
                stats_data.insert(
                    stats_data.len(),
                    Datum::int("Max SP", &mut p.max_stamina, 0, i32::MAX),
                );
            }
            if let Some(b) = self.blessings.as_mut() {
                stats_data.insert(stats_data.len(), Datum::separator());
                stats_data.append(&mut vec![
                    Datum::byte("Scadutree Blessing", &mut b.scadutree, 0, 20),
                    Datum::byte("Revered Spirit Ash", &mut b.revered_spirit_ash, 0, 10),
                ]);
            }
            stats_data.into_iter()
        })
    }

    fn read(&mut self) {
        self.stats = self.stats_ptr.read();
        self.points = self.points_ptr.read();
        if let Some(ptr) = &self.blessings_ptr {
            self.blessings = ptr.read();
        }
    }

    fn write(&mut self) {
        if let Some(stats) = self.stats.clone() {
            self.stats_ptr.write(stats);
        }
        if let Some(h) = self.points.clone() {
            self.points_ptr.write(h);
        }
        if let Some(ptr) = &self.blessings_ptr {
            if let Some(blessings) = self.blessings.clone() {
                ptr.write(blessings);
            }
        }
    }

    fn clear(&mut self) {
        self.stats = None;
        self.blessings = None;
    }
}

pub(crate) fn character_stats_edit(
    character_stats: PointerChain<CharacterStats>,
    character_points: PointerChain<CharacterPoints>,
    character_blessings: Option<PointerChain<CharacterBlessings>>,
    key_open: Option<Key>,
    key_close: Key,
) -> Box<dyn Widget> {
    Box::new(StatsEditor::new(
        CharacterStatsEdit {
            stats_ptr: character_stats,
            points_ptr: character_points,
            blessings_ptr: character_blessings,
            stats: None,
            points: None,
            blessings: None,
        },
        key_open,
        Some(key_close),
    ))
}
