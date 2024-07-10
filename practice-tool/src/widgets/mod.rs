pub(crate) mod action_freeze;
pub(crate) mod character_stats;
pub(crate) mod cycle_speed;
pub(crate) mod deathcam;
pub(crate) mod flag;
pub(crate) mod group;
pub(crate) mod item_spawn;
pub(crate) mod label;
pub(crate) mod multiflag;
pub(crate) mod nudge_pos;
pub(crate) mod position;
pub(crate) mod quitout;
pub(crate) mod runes;
pub(crate) mod savefile_manager;
pub(crate) mod target;
pub(crate) mod warp;

pub(crate) fn string_match(needle: &str, haystack: &str) -> bool {
    let needle = needle.chars().flat_map(char::to_lowercase);
    let mut haystack = haystack.chars().flat_map(char::to_lowercase);

    'o: for c in needle {
        for d in &mut haystack {
            if c == d {
                continue 'o;
            }
        }
        return false;
    }
    true
}
