use std::mem;

use hudhook::tracing::info;
use imgui::sys::{igGetCursorPosX, igGetCursorPosY, igGetWindowPos, igSetNextWindowPos, ImVec2};
use imgui::{Condition, InputText, WindowFlags};
use libeldenring::prelude::*;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::{scaling_factor, Widget, BUTTON_HEIGHT, BUTTON_WIDTH};

use super::string_match;

type WarpFunc = extern "system" fn(u64, u64, u32);

const POPUP_TAG: &str = "##warp";

#[derive(Debug)]
pub(crate) struct Warp {
    label: String,
    label_close: String,
    hotkey_close: Key,
    warp_ptr: usize,
    arg1: PointerChain<u64>,
    arg2: PointerChain<u64>,
    current_grace: usize,
    filter_string: String,
    filter_list: [bool; GRACES.len()],
}

impl Warp {
    pub(crate) fn new(
        warp_ptr: usize,
        arg1: PointerChain<u64>,
        arg2: PointerChain<u64>,
        hotkey_close: Key,
    ) -> Self {
        let label_close = format!("Close ({hotkey_close})");
        Warp {
            label: "Warp to Grace".to_string(),
            label_close,
            hotkey_close,
            warp_ptr,
            arg1,
            arg2,
            current_grace: 0,
            filter_string: String::new(),
            filter_list: [true; GRACES.len()],
        }
    }

    fn warp(&mut self) {
        let warp_fn: WarpFunc = unsafe { mem::transmute(self.warp_ptr) };
        let arg1 = self.arg1.read();
        let arg2 = self.arg2.read();

        info!("{:?} {:?}", arg1, arg2);

        if let (Some(arg1), Some(arg2)) = (arg1, arg2) {
            warp_fn(arg1, arg2, GRACES[self.current_grace].1 - 0x3e8);
        }
    }
}

impl Widget for Warp {
    fn render(&mut self, ui: &imgui::Ui) {
        let scale = scaling_factor(ui);
        let button_width = BUTTON_WIDTH * scale;
        let button_height = BUTTON_HEIGHT;

        let (x, y) = unsafe {
            let mut wnd_pos = ImVec2::default();
            igGetWindowPos(&mut wnd_pos);
            (igGetCursorPosX() + wnd_pos.x, igGetCursorPosY() + wnd_pos.y)
        };

        if ui.button_with_size(&self.label, [button_width, button_height]) {
            ui.open_popup(POPUP_TAG);
        }

        unsafe {
            igSetNextWindowPos(
                ImVec2::new(x + 200. * scale, y),
                Condition::Always as i8 as _,
                ImVec2::new(0., 0.),
            )
        };
        if let Some(_token) = ui
            .modal_popup_config(POPUP_TAG)
            .flags(
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::ALWAYS_AUTO_RESIZE,
            )
            .begin_popup()
        {
            let _tok = ui.push_item_width(-1.);

            if InputText::new(ui, "##warp-filter", &mut self.filter_string)
                .hint("Filter...")
                .build()
            {
                GRACES.iter().enumerate().for_each(|(idx, (grace, _))| {
                    self.filter_list[idx] =
                        self.filter_string.is_empty() || string_match(&self.filter_string, grace)
                });

                if !self.filter_list[self.current_grace] {
                    self.current_grace = self.filter_list.iter().position(|f| *f).unwrap_or(0);
                }
            }

            let _tok = ui.push_item_width(-1.);
            if let Some(_combo) = ui.begin_combo("##warp-graces", GRACES[self.current_grace].0) {
                for (idx, (grace, _)) in
                    GRACES.iter().enumerate().filter(|(idx, _)| self.filter_list[*idx])
                {
                    let selected = idx == self.current_grace;
                    if selected {
                        ui.set_item_default_focus();
                    }

                    if ui.selectable_config(grace).selected(selected).build() {
                        self.current_grace = idx;
                    }
                }
            }

            let _tok = ui.push_item_width(-1.);
            if ui.button_with_size("Warp", [400., button_height]) {
                self.warp();
            }

            let _tok = ui.push_item_width(-1.);
            if ui.button_with_size(&self.label_close, [400., button_height])
                || (self.hotkey_close.is_pressed(ui) && !ui.is_any_item_active())
            {
                ui.close_current_popup();
            }
        }
    }
}

const GRACES: &[(&str, u32)] = &[
    ("[Abyssal Woods] Abyssal Woods", 2050422950),
    ("[Abyssal Woods] Church Ruins", 2053412950),
    ("[Abyssal Woods] Divided Falls", 2048432951),
    ("[Abyssal Woods] Forsaken Graveyard", 2052432950),
    ("[Abyssal Woods] Woodland Trail", 2051422950),
    ("[Academy of Raya Lucaria] Church of the Cuckoo", 14002952),
    ("[Academy of Raya Lucaria] Debate Parlor", 14002951),
    ("[Academy of Raya Lucaria] Raya Lucaria Grand Library", 14002950),
    ("[Academy of Raya Lucaria] Schoolhouse Classroom", 14002953),
    ("[Ainsel River] Ainsel River Downstream", 12012953),
    ("[Ainsel River] Ainsel River Sluice Gate", 12012952),
    ("[Ainsel River] Ainsel River Well Depths", 12012951),
    ("[Ainsel River] Astel - Naturalborn of the Void", 12042950),
    ("[Ainsel River] Dragonkin Soldier of Nokstella", 12012950),
    ("[Ainsel River Main] Ainsel River Main", 12012954),
    ("[Ainsel River Main] Nokstella - Eternal City", 12012955),
    ("[Ainsel River Main] Nokstella Waterfall Basin", 12012959),
    ("[Altus Plateau] Abandoned Coffin", 1037512950),
    ("[Altus Plateau] Altus Highway Junction", 1039512950),
    ("[Altus Plateau] Altus Plateau", 1038502952),
    ("[Altus Plateau] Altus Tunnel", 32052950),
    ("[Altus Plateau] Bower of Bounty", 1040532950),
    ("[Altus Plateau] Castellan's Hall", 1039542952),
    ("[Altus Plateau] Erdtree-Gazing Hill", 1038512950),
    ("[Altus Plateau] Forest-Spanning Greatbridge", 1040522950),
    ("[Altus Plateau] Old Altus Tunnel", 32042950),
    ("[Altus Plateau] Perfumer's Grotto", 31182950),
    ("[Altus Plateau] Rampartside Path", 1041522951),
    ("[Altus Plateau] Road of Iniquity Side Path", 1040542950),
    ("[Altus Plateau] Sage's Cave", 31192950),
    ("[Altus Plateau] Sainted Hero's Grave", 30082950),
    ("[Altus Plateau] Shaded castle Inner Gate", 1039542951),
    ("[Altus Plateau] Shaded Castle Ramparts", 1039542950),
    ("[Altus Plateau] Unsightly Catacombs", 30122950),
    ("[Altus Plateau] Windmill Heights", 1042552950),
    ("[Altus Plateau] Windmill Village", 1041542950),
    ("[Ancient Ruins of Rauh] Ancient Ruins- Grand Stairway", 2044452950),
    ("[Ancient Ruins of Rauh] Church of the Bud", 2044452951),
    ("[Ancient Ruins of Rauh] Church of the Bud - Main Entrance", 2044462950),
    ("[Ancient Ruins of Rauh] Rauh Ancient Ruins- East", 2046472950),
    ("[Ancient Ruins of Rauh] Rauh Ancient Ruins- West", 2045452951),
    ("[Ancient Ruins of Rauh] Viaduct Minor Tower", 2047472950),
    ("[Bellum Highway] Bellum Church", 1036492950),
    ("[Bellum Highway] Church of Inhibition", 1037492950),
    ("[Bellum Highway] East Raya Lucaria Gate", 1036482950),
    ("[Bellum Highway] Frenzied Flame Village Outskirts", 1038482950),
    ("[Belurat - Tower Settlement] Belurat- Tower Settlement", 20002951),
    ("[Belurat - Tower Settlement] Small Private Altar", 20002952),
    ("[Belurat - Tower Settlement] Stagefront", 20002953),
    ("[Belurat - Tower Settlement] Theatre of the Divine Beast", 20002950),
    ("[Caelid] Abandoned Cave", 31202950),
    ("[Caelid] Caelem Ruins", 1047402950),
    ("[Caelid] Caelid Catacombs", 30152950),
    ("[Caelid] Caelid Highway South", 1048372950),
    ("[Caelid] Cathedral of Dragon Communion", 1048362950),
    ("[Caelid] Chair-Crypt of Sellia", 1049392951),
    ("[Caelid] Chamber Outside the Plaza", 1051362951),
    ("[Caelid] Church of the Plague", 1050382950),
    ("[Caelid] Deep Siofra Well", 1048402950),
    ("[Caelid] Fort Gael North", 1047392950),
    ("[Caelid] Gael Tunnel", 32072950),
    ("[Caelid] Gaol Cave", 31212950),
    ("[Caelid] Impassable Greatbridge", 1050362950),
    ("[Caelid] Minor Erdtree Catacombs", 30142950),
    ("[Caelid] Rear Gael Tunnel Entrance", 32072951),
    ("[Caelid] Redmane Castle Plaza", 1051362950),
    ("[Caelid] Rotview Balcony", 1046402951),
    ("[Caelid] Sellia Backstreets", 1049392950),
    ("[Caelid] Sellia Crystal Tunnel", 32082950),
    ("[Caelid] Sellia Under-Stair", 1049392952),
    ("[Caelid] Smoldering Church", 1046402950),
    ("[Caelid] Smoldering Wall", 1048392950),
    ("[Caelid] Southern Aeonia Swamp Bank", 1049372950),
    ("[Caelid] Starscourge Radahn", 1052382950),
    ("[Caelid] War-Dead Catacombs", 30162950),
    ("[Capital Outskirts] Auriza Hero's Grave", 30102950),
    ("[Capital Outskirts] Auriza Side Tomb", 30132950),
    ("[Capital Outskirts] Capital Rampart", 1045522950),
    ("[Capital Outskirts] Divine Tower of West Altus", 34122950),
    ("[Capital Outskirts] Divine Tower of West Altus: Gate", 34122952),
    ("[Capital Outskirts] Hermit Merchant's Shack", 1043532950),
    ("[Capital Outskirts] Minor Erdtree Church", 1043502950),
    ("[Capital Outskirts] Outer Wall Battleground", 1043532951),
    ("[Capital Outskirts] Outer Wall Phantom Tree", 1042512950),
    ("[Capital Outskirts] Sealed Tunnel", 34122951),
    ("[Castle Ensis] Castle Ensis Checkpoint", 2047442951),
    ("[Castle Ensis] Castle-Lord's Chamber", 2048442951),
    ("[Castle Ensis] Ensis Moongazing Grounds", 2048442950),
    ("[Cerulean Coast] Cerulean Coast", 2048392950),
    ("[Cerulean Coast] Cerulean Coast Cross", 2048372950),
    ("[Cerulean Coast] Cerulean Coast West", 2046392950),
    ("[Cerulean Coast] Finger Ruins of Rhia", 2050382950),
    ("[Cerulean Coast] The Fissure", 2047352950),
    ("[Charo's Hidden Grave] Charo's Hidden Grave", 2048392951),
    ("[Charo's Hidden Grave] Lamenter's Gaol", 41022950),
    ("[Consecrated Snowfield] Apostate Derelict", 1047582950),
    ("[Consecrated Snowfield] Cave of the Forlorn", 31122950),
    ("[Consecrated Snowfield] Consecrated Snowfield", 1049542950),
    ("[Consecrated Snowfield] Consecrated Snowfield Catacombs", 30192950),
    ("[Consecrated Snowfield] Inner Consecrated Snowfield", 1049552950),
    ("[Consecrated Snowfield] Ordina - Liturgical Town", 1048572950),
    ("[Consecrated Snowfield] Yelough Anix Tunnel", 32112950),
    ("[Crumbling Farum Azula] Beside the Great Bridge", 13002960),
    ("[Crumbling Farum Azula] Crumbling Beast Grave", 13002953),
    ("[Crumbling Farum Azula] Crumbling Beast Grave Depths", 13002954),
    ("[Crumbling Farum Azula] Dragonlord Placidusax", 13002951),
    ("[Crumbling Farum Azula] Dragon Temple", 13002956),
    ("[Crumbling Farum Azula] Dragon Temple Altar", 13002952),
    ("[Crumbling Farum Azula] Dragon Temple Lift", 13002958),
    ("[Crumbling Farum Azula] Dragon Temple Rooftop", 13002959),
    ("[Crumbling Farum Azula] Dragon Temple Transept", 13002957),
    ("[Crumbling Farum Azula] Maliketh - the Black Blade", 13002950),
    ("[Crumbling Farum Azula] Tempest-Facing Balcony", 13002955),
    ("[Deeproot Depths] Across the Roots", 12032955),
    ("[Deeproot Depths] Deeproot Depths", 12032953),
    ("[Deeproot Depths] Great Waterfall Crest", 12032952),
    ("[Deeproot Depths] Prince of Death's Throne", 12032950),
    ("[Deeproot Depths] Root-Facing Cliffs", 12032951),
    ("[Deeproot Depths] The Nameless Eternal City", 12032954),
    ("[Elphael - Brace of the Haligtree] Drainage Channel", 15002953),
    ("[Elphael - Brace of the Haligtree] Elphael Inner Wall", 15002952),
    ("[Elphael - Brace of the Haligtree] Haligtree Roots", 15002954),
    ("[Elphael - Brace of the Haligtree] Malenia- Goddess of Rot", 15002950),
    ("[Elphael - Brace of the Haligtree] Prayer Room", 15002951),
    ("[Enir-Ilim] Cleansing Chamber Anteroom", 20012955),
    ("[Enir-Ilim] Divine Gate Front Staircase", 20012956),
    ("[Enir-Ilim] Enir-Ilim: Outer Wall", 20012952),
    ("[Enir-Ilim] First Rise", 20012953),
    ("[Enir-Ilim] Gate of Divinity", 20012950),
    ("[Enir-Ilim] Spiral Rise", 20012954),
    ("[Flame Peak] Church of Repose", 1051532950),
    ("[Flame Peak] Giant-Conquering Hero's Grave", 30172950),
    ("[Flame Peak] Giants' Mountaintop Catacombs", 30182950),
    ("[Foot of the Jagged Peak] Foot of the Jagged Peak", 2052402950),
    ("[Foot of the Jagged Peak] Grand Altar of Dragon Communion", 2049392950),
    ("[Forbidden Lands] Divine Tower of East Altus", 34142951),
    ("[Forbidden Lands] Divine Tower of East Altus: Gate", 34142950),
    ("[Forbidden Lands] Forbidden Lands", 1047512950),
    ("[Forbidden Lands] Grand Lift of Rold", 1049532951),
    ("[Forbidden Lands] Hidden Path to the Haligtree", 30202950),
    ("[Gravesite Plain] Belurat Gaol", 41002950),
    ("[Gravesite Plain] Castle Front", 2047442950),
    ("[Gravesite Plain] Cliffroad Terminus", 2045412950),
    ("[Gravesite Plain] Dragon's Pit", 43012950),
    ("[Gravesite Plain] Dragon's Pit Terminus", 43012951),
    ("[Gravesite Plain] Ellac River Cave", 2047432950),
    ("[Gravesite Plain] Ellac River Downstream", 2047412951),
    ("[Gravesite Plain] Fog Rift Catacombs", 40002950),
    ("[Gravesite Plain] Gravesite Plain", 2046402950),
    ("[Gravesite Plain] Greatbridge - North", 2046442950),
    ("[Gravesite Plain] Main Gate Cross", 2045422950),
    ("[Gravesite Plain] Pillar Path Cross", 2048432950),
    ("[Gravesite Plain] Pillar Path Waypoint", 2048422950),
    ("[Gravesite Plain] Rivermouth Cave", 43002950),
    ("[Gravesite Plain] Ruined Forge Lava Intake", 42002950),
    ("[Gravesite Plain] Scorched Ruins", 2047412950),
    ("[Gravesite Plain] Three-Path Cross", 2046422950),
    ("[Greyoll's Dragonbarrow] Bestial Sanctum", 1051432950),
    ("[Greyoll's Dragonbarrow] Divine Tower of Caelid: Basement", 34132951),
    ("[Greyoll's Dragonbarrow] Divine Tower of Caelid: Center", 34132952),
    ("[Greyoll's Dragonbarrow] Dragonbarrow Cave", 31102950),
    ("[Greyoll's Dragonbarrow] Dragonbarrow Fork", 1050402950),
    ("[Greyoll's Dragonbarrow] Dragonbarrow West", 1048402951),
    ("[Greyoll's Dragonbarrow] Farum Greatbridge", 1052422950),
    ("[Greyoll's Dragonbarrow] Fort Faroth", 1051392950),
    ("[Greyoll's Dragonbarrow] Isolated Divine Tower", 34152950),
    ("[Greyoll's Dragonbarrow] Isolated Merchant's Shack", 1048412950),
    ("[Greyoll's Dragonbarrow] Lenne's Rise", 1052412950),
    ("[Greyoll's Dragonbarrow] Sellia Hideaway", 31112950),
    ("[Jagged Peak] Jagged Peak Mountainside", 2053392950),
    ("[Jagged Peak] Jagged Peak Summit", 2054392950),
    ("[Jagged Peak] Rest of the Dread Dragon", 2055392950),
    ("[Lake of Rot] Grand Cloister", 12012958),
    ("[Lake of Rot] Lake of Rot Shoreside", 12012956),
    ("[Leyndell - Ashen Capital] Divine Bridge", 11052955),
    ("[Leyndell - Ashen Capital] East Capital Rampart", 11052952),
    ("[Leyndell - Ashen Capital] Elden Throne", 11052950),
    ("[Leyndell - Ashen Capital] Erdtree Sanctuary", 11052951),
    ("[Leyndell - Ashen Capital] Leyndell- Capital of Ash", 11052953),
    ("[Leyndell - Ashen Capital] Queen's Bedchamber", 11052954),
    ("[Leyndell - Royal Capital] Avenue Balcony", 11002954),
    ("[Leyndell - Royal Capital] Divine Bridge", 11002959),
    ("[Leyndell - Royal Capital] East Capital Rampart", 11002952),
    ("[Leyndell - Royal Capital] Elden Throne", 11002950),
    ("[Leyndell - Royal Capital] Erdtree Sanctuary", 11002951),
    ("[Leyndell - Royal Capital] Fortified Manor- First Floor", 11002958),
    ("[Leyndell - Royal Capital] Lower Capital Church", 11002953),
    ("[Leyndell - Royal Capital] Queen's Bedchamber", 11002957),
    ("[Leyndell - Royal Capital] West Capital Rampart", 11002955),
    ("[Limgrave] Agheel Lake North", 1043372950),
    ("[Limgrave] Agheel Lake South", 1044352950),
    ("[Limgrave] Artist's Shack", 1044382950),
    ("[Limgrave] Church of Dragon Communion", 1041352950),
    ("[Limgrave] Church of Elleh", 1042362950),
    ("[Limgrave] Coastal Cave", 31152950),
    ("[Limgrave] Fort Haight West", 1045362950),
    ("[Limgrave] Gatefront Ruins", 1042372950),
    ("[Limgrave] Groveside Cave", 31032950),
    ("[Limgrave] Highroad Cave", 31172950),
    ("[Limgrave] Limgrave Tunnels", 32012950),
    ("[Limgrave] Mistwood Outskirts", 1044372950),
    ("[Limgrave] Murkwater Catacombs", 30042950),
    ("[Limgrave] Murkwater Cave", 31002950),
    ("[Limgrave] Murkwater Coast", 1043382950),
    ("[Limgrave] Seaside Ruins", 1043352950),
    ("[Limgrave] Stormfoot Catacombs", 30022950),
    ("[Limgrave] Summonwater Village Outskirts", 1044392950),
    ("[Limgrave] The First Step", 1042362951),
    ("[Limgrave] Third Church of Marika", 1046382950),
    ("[Limgrave] Waypoint Ruins Cellar", 1044362950),
    ("[Liurnia of the Lakes] Academy Crystal Cave", 31062950),
    ("[Liurnia of the Lakes] Academy Gate Town", 1037442950),
    ("[Liurnia of the Lakes] Artist's Shack", 1038452950),
    ("[Liurnia of the Lakes] Behind Caria Manor", 1036502950),
    ("[Liurnia of the Lakes] Black Knife Catacombs", 30052950),
    ("[Liurnia of the Lakes] Boilprawn Shack", 1036432950),
    ("[Liurnia of the Lakes] Church of Vows", 1037462950),
    ("[Liurnia of the Lakes] Cliffbottom Catacombs", 30062950),
    ("[Liurnia of the Lakes] Converted Tower", 1034432950),
    ("[Liurnia of the Lakes] Crystalline Woods", 1034462950),
    ("[Liurnia of the Lakes] Divine Tower of Liurnia", 34112952),
    ("[Liurnia of the Lakes] Eastern Liurnia Lake Shore", 1038452951),
    ("[Liurnia of the Lakes] Eastern Tableland", 1038462950),
    ("[Liurnia of the Lakes] East Gate Bridge Trestle", 1035472950),
    ("[Liurnia of the Lakes] Fallen Ruins of the Lake", 1036432951),
    ("[Liurnia of the Lakes] Folly on the Lake", 1035432950),
    ("[Liurnia of the Lakes] Foot of the Four Belfries", 1033462950),
    ("[Liurnia of the Lakes] Gate Town Bridge", 1038432950),
    ("[Liurnia of the Lakes] Gate Town North", 1036452950),
    ("[Liurnia of the Lakes] Grand Lift of Dectus", 1038502950),
    ("[Liurnia of the Lakes] Jarburg", 1039442950),
    ("[Liurnia of the Lakes] Kingsrealm Ruins", 1034482950),
    ("[Liurnia of the Lakes] Lake-Facing Cliffs", 1039402950),
    ("[Liurnia of the Lakes] Lakeside Crystal Cave", 31052950),
    ("[Liurnia of the Lakes] Laskyar Ruins", 1038412950),
    ("[Liurnia of the Lakes] Liurnia Highway North", 1039422950),
    ("[Liurnia of the Lakes] Liurnia Highway South", 1039412950),
    ("[Liurnia of the Lakes] Liurnia Lake Shore", 1038402950),
    ("[Liurnia of the Lakes] Liurnia Tower Bridge", 34112951),
    ("[Liurnia of the Lakes] Main Academy Gate", 1035462950),
    ("[Liurnia of the Lakes] Main Caria Manor Gate", 1035502953),
    ("[Liurnia of the Lakes] Manor Lower Level", 1035502951),
    ("[Liurnia of the Lakes] Manor Upper Level", 1035502950),
    ("[Liurnia of the Lakes] Mausoleum Compound", 1037482950),
    ("[Liurnia of the Lakes] Ranni's Chamber", 1034502951),
    ("[Liurnia of the Lakes] Ranni's Rise", 1034502950),
    ("[Liurnia of the Lakes] Ravine-Veiled Village", 1038502951),
    ("[Liurnia of the Lakes] Raya Lucaria Crystal Tunnel", 32022950),
    ("[Liurnia of the Lakes] Revenger's Shack", 1033442950),
    ("[Liurnia of the Lakes]Road's End Catacombs", 30032950),
    ("[Liurnia of the Lakes] Road to the Manor", 1034492950),
    ("[Liurnia of the Lakes] Royal Moongazing Grounds", 1035502952),
    ("[Liurnia of the Lakes] Ruined Labyrinth", 1038472950),
    ("[Liurnia of the Lakes] Scenic Isle", 1037422950),
    ("[Liurnia of the Lakes] Slumbering Wolf's Shack", 1036412950),
    ("[Liurnia of the Lakes] Sorcerer's Isle", 1034472951),
    ("[Liurnia of the Lakes] South Raya Lucaria Gate", 1035452950),
    ("[Liurnia of the Lakes] Stillwater Cave", 31042950),
    ("[Liurnia of the Lakes] Study Hall Entrance", 34112950),
    ("[Liurnia of the Lakes] Temple Quarter", 1034442950),
    ("[Liurnia of the Lakes] The Four Belfries", 1033472950),
    ("[Liurnia of the Lakes] The Ravine", 1036492951),
    ("[Liurnia of the Lakes] Village of the Albinaurics", 1034422950),
    ("[Midra's Manse] Discussion Chamber", 28002950),
    ("[Midra's Manse] Manse Hall", 28002951),
    ("[Midra's Manse] Midra's Library", 28002952),
    ("[Midra's Manse] Second Floor Chamber", 28002953),
    ("[Miquella's Haligtree] Haligtree Canopy", 15002956),
    ("[Miquella's Haligtree] Haligtree Promenade", 15002955),
    ("[Miquella's Haligtree] Haligtree Town", 15002957),
    ("[Miquella's Haligtree] Haligtree Town Plaza", 15002958),
    ("[Mohgwyn Palace] Cocoon of the Empyrean", 12052950),
    ("[Mohgwyn Palace] Dynasty Mausoleum Entrance", 12052952),
    ("[Mohgwyn Palace] Dynasty Mausoleum Midpoint", 12052953),
    ("[Mohgwyn Palace] Palace Approach Ledge-Road", 12052951),
    ("[Moonlight Altar] Altar South", 1033402950),
    ("[Moonlight Altar] Cathedral of Manus Celes", 1035422950),
    ("[Moonlight Altar] Moonlight Altar", 1034412950),
    ("[Mountaintops of the Giants] Ancient Snow Valley Ruins", 1051562950),
    ("[Mountaintops of the Giants] Castle Sol Main Gate", 1051572951),
    ("[Mountaintops of the Giants] Castle Sol Rooftop", 1051572953),
    ("[Mountaintops of the Giants] Church of the Eclipse", 1051572952),
    ("[Mountaintops of the Giants] Fire Giant", 1053522950),
    ("[Mountaintops of the Giants] First Church of Marika", 1054552950),
    ("[Mountaintops of the Giants] Foot of the Forge", 1052532950),
    ("[Mountaintops of the Giants] Forge of the Giants", 1054532950),
    ("[Mountaintops of the Giants] Freezing Lake", 1052572950),
    ("[Mountaintops of the Giants] Giant's Gravepost", 1052542950),
    ("[Mountaintops of the Giants] Snow Valley Ruins Overlook", 1051572950),
    ("[Mountaintops of the Giants] Spiritcaller's Cave", 31222950),
    ("[Mountaintops of the Giants] Whiteridge Road", 1052562950),
    ("[Mountaintops of the Giants] Zamor Ruins", 1049532950),
    ("[Mt. Gelmir] Bridge of Iniquity", 1039532950),
    ("[Mt. Gelmir] Craftman's Shack", 1036522950),
    ("[Mt. Gelmir] First Mt. Gelmir Campsite", 1038542950),
    ("[Mt. Gelmir] Gelmir Hero's Grave", 30092950),
    ("[Mt. Gelmir] Ninth Mt. Gelmir Campsite", 1036542951),
    ("[Mt. Gelmir] Primeval Sorcerer Azur", 1037532950),
    ("[Mt. Gelmir] Road of Iniquity", 1036542952),
    ("[Mt. Gelmir] Seethewater Cave", 31072950),
    ("[Mt. Gelmir] Seethewater River", 1037522951),
    ("[Mt. Gelmir] Seethewater Terminus", 1035532950),
    ("[Mt. Gelmir] Volcano Cave", 31092950),
    ("[Mt. Gelmir] Wyndham Catacombs", 30072950),
    ("[Nokron - Eternal City] Ancestral Woods", 12022956),
    ("[Nokron - Eternal City] Aqueduct-Facing Cliffs", 12022957),
    ("[Nokron - Eternal City] Great Waterfall Basin", 12022950),
    ("[Nokron - Eternal City] Mimic Tear", 12022951),
    ("[Nokron - Eternal City] Night's Sacred Ground", 12022958),
    ("[Nokron - Eternal City] Nokron- Eternal City", 12072951),
    ("[Rauh Base] Ancient Ruins Base", 2048472950),
    ("[Rauh Base] Ravine North", 2045472950),
    ("[Rauh Base] Scorpion River Catacombs", 40012950),
    ("[Rauh Base] Taylew's Ruined Forge", 42032950),
    ("[Rauh Base] Temple Town Ruins", 2045462950),
    ("[Roundtable Hold] Table of Lost Grace", 11102950),
    ("[Ruin-Strewn Precipice] Magma Wyrm Makar", 39202950),
    ("[Ruin-Strewn Precipice] Ruin-Strewn Precipice", 39202951),
    ("[Ruin-Strewn Precipice] Ruin-Strewn Precipice Overlook", 39202952),
    ("[Scadu Altus] Behind the Fort of Reprimand", 2049432952),
    ("[Scadu Altus] Bonny Gaol", 41012950),
    ("[Scadu Altus] Bonny Village", 2049442951),
    ("[Scadu Altus] Bridge Leading to the Village", 2051442950),
    ("[Scadu Altus] Castle Watering Hole", 2049472950),
    ("[Scadu Altus] Cathedral of Manus Metyr", 2051452950),
    ("[Scadu Altus] Church District Highroad", 2051472950),
    ("[Scadu Altus] Darklight Catacombs", 40022950),
    ("[Scadu Altus] Finger Birthing Grounds", 25002950),
    ("[Scadu Altus] Fort of Reprimand", 2049432951),
    ("[Scadu Altus] Highroad Cross", 2048452950),
    ("[Scadu Altus] Moorth Highway - South", 2049432950),
    ("[Scadu Altus] Moorth Ruins", 2049442950),
    ("[Scadu Altus] Recluses' River Downstream", 2050442950),
    ("[Scadu Altus] Recluses' River Upstream", 2050452950),
    ("[Scadu Altus] Ruined Forge of Starfall Past", 42022950),
    ("[Scadu Altus] Scadu Altus- West", 2047452950),
    ("[Scadu Altus] Scaduview Cross", 2050432950),
    ("[Scaduview] Fingerstone Hill", 2051482950),
    ("[Scaduview] Hinterland", 2050482950),
    ("[Scaduview] Hinterland Bridge", 2051482951),
    ("[Scaduview] Scadutree Base", 2050482951),
    ("[Scaduview] Scaduview", 2049482950),
    ("[Scaduview] Shadow Keep - Back Gate", 2049482951),
    ("[Shadow Keep - Church District] Church District Entrance", 21002956),
    ("[Shadow Keep - Church District] Sunken Chapel", 21002957),
    ("[Shadow Keep - Church District] Tree-Worship Passage", 21002958),
    ("[Shadow Keep - Church District] Tree-Worship Sanctum", 21002959),
    ("[Shadow Keep] Main Gate Plaza", 21002951),
    ("[Shadow Keep] Shadow Keep Main Gate", 21002952),
    ("[Siofra River] Below the Well", 12022959),
    ("[Siofra River] Siofra River Bank", 12022953),
    ("[Siofra River] Siofra River Well Depths", 12072950),
    ("[Siofra River] Worshippers' Woods", 12022954),
    ("[Specimen Storehouse] Dark Chamber Entrance", 21012954),
    ("[Specimen Storehouse] Messmer's Dark Chamber", 21012950),
    ("[Specimen Storehouse] Storehouse - Back Section", 21012956),
    ("[Specimen Storehouse] Storehouse - First Floor", 21012951),
    ("[Specimen Storehouse] Storehouse - Fourth Floor", 21012952),
    ("[Specimen Storehouse] Storehouse - Loft", 21012957),
    ("[Specimen Storehouse] Storehouse - Seventh Floor", 21012953),
    ("[Specimen Storehouse] West Rampart", 21022951),
    ("[Stone Coffin Fissure] Fissure Cross", 22002952),
    ("[Stone Coffin Fissure] Fissure Depths", 22002954),
    ("[Stone Coffin Fissure] Fissure Waypoint", 22002953),
    ("[Stone Coffin Fissure] Garden of Deep Purple", 22002950),
    ("[Stone Coffin Fissure] Stone Coffin Fissure", 22002951),
    ("[Stone Platform] Fractured Marika", 19002950),
    ("[Stormhill] Castleward Tunnel", 10002952),
    ("[Stormhill] Deathtouched Catacombs", 30112950),
    ("[Stormhill] Divine Tower of Limgrave", 34102952),
    ("[Stormhill] Limgrave Tower Bridge", 34102950),
    ("[Stormhill] Margit - the Fell Omen", 10002951),
    ("[Stormhill] Saintsbridge", 1043392950),
    ("[Stormhill] Stormhill Shack", 1041382950),
    ("[Stormhill] Warmaster's Shack", 1042382950),
    ("[Stormveil Castle] Gateside Chamber", 10002953),
    ("[Stormveil Castle] Godrick the Grafted", 10002950),
    ("[Stormveil Castle] Liftside Chamber", 10002956),
    ("[Stormveil Castle] Rampart Tower", 10002955),
    ("[Stormveil Castle] Secluded Cell", 10002957),
    ("[Stormveil Castle] Stormveil Cliffside", 10002954),
    ("[Stormveil Castle] Stormveil Main Gate", 10002958),
    ("[Stranded Graveyard] Cave of Knowledge", 18002950),
    ("[Stranded Graveyard] Stranded Graveyard", 18002951),
    ("[Subterranean Shunning-Grounds] Cathedral of the Forsaken", 35002950),
    ("[Subterranean Shunning-Grounds] Forsaken Depths", 35002952),
    ("[Subterranean Shunning-Grounds] Frenzied Flame Proscription", 35002954),
    ("[Subterranean Shunning-Grounds] Leyndell Catacombs", 35002953),
    ("[Subterranean Shunning-Grounds] Underground Roadside", 35002951),
    ("[Swamp of Aeonia] Aeonia Swamp Shore", 1048382950),
    ("[Swamp of Aeonia] Astray from Caelid Highway North", 1048382951),
    ("[Swamp of Aeonia] Heart of Aeonia", 1049382950),
    ("[Swamp of Aeonia] Inner Aeonia", 1049382951),
    ("[Volcano Manor] Abductor Virgin", 16002962),
    ("[Volcano Manor] Audience Pathway", 16002960),
    ("[Volcano Manor] Guest Hall", 16002954),
    ("[Volcano Manor] Prison Town Church", 16002953),
    ("[Volcano Manor] Rykard - Lord of Blasphemy", 16002950),
    ("[Volcano Manor] Subterranean Inquisition Chamber", 16002964),
    ("[Volcano Manor] Temple of Eiglay", 16002951),
    ("[Volcano Manor] Volcano Manor", 16002952),
    ("[Weeping Peninsula] Ailing Village Outskirts", 1044332952),
    ("[Weeping Peninsula] Behind the Castle", 1043312951),
    ("[Weeping Peninsula] Beside the Crater-Pocked Glade", 1045332950),
    ("[Weeping Peninsula] Beside the Rampart Gaol", 1043312952),
    ("[Weeping Peninsula] Bridge of Sacrifice", 1044342950),
    ("[Weeping Peninsula] Castle Morne Lift", 1043312950),
    ("[Weeping Peninsula] Castle Morne Rampart", 1044332950),
    ("[Weeping Peninsula] Church of Pilgrimage", 1043342950),
    ("[Weeping Peninsula] Earthbore Cave", 31012950),
    ("[Weeping Peninsula] Fourth Church of Marika", 1041332950),
    ("[Weeping Peninsula] Impaler's Catacombs", 30012950),
    ("[Weeping Peninsula] Isolated Merchant's Shack", 1041322950),
    ("[Weeping Peninsula] Morne Moangrave", 1043302950),
    ("[Weeping Peninsula] Morne Tunnel", 32002950),
    ("[Weeping Peninsula] South of the Lookout Tower", 1044332951),
    ("[Weeping Peninsula] Tombsward Catacombs", 30002950),
    ("[Weeping Peninsula] Tombsward Cave", 31022950),
    ("[Weeping Peninsula] Weeping Evergaol", 1042332950),
];
