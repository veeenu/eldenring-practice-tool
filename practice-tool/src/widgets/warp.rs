use std::mem;

use hudhook::tracing::info;
use imgui::sys::{igGetCursorPosX, igGetCursorPosY, igGetWindowPos, igSetNextWindowPos, ImVec2};
use imgui::{Condition, InputText, Key, WindowFlags};
use libeldenring::prelude::*;

use super::{scaling_factor, string_match, Widget, BUTTON_HEIGHT, BUTTON_WIDTH};

type WarpFunc = extern "system" fn(u64, u64, u32);

const POPUP_TAG: &str = "##warp";

#[derive(Debug)]
pub(crate) struct Warp {
    label: String,
    warp_ptr: usize,
    arg1: PointerChain<u64>,
    arg2: PointerChain<u64>,
    current_grace: usize,
    filter_string: String,
    filter_list: [bool; GRACES.len()],
}

impl Warp {
    pub(crate) fn new(warp_ptr: usize, arg1: PointerChain<u64>, arg2: PointerChain<u64>) -> Self {
        Warp {
            label: "Warp".to_string(),
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

        let (x, y) = unsafe {
            let mut wnd_pos = ImVec2::default();
            igGetWindowPos(&mut wnd_pos);
            (igGetCursorPosX() + wnd_pos.x, igGetCursorPosY() + wnd_pos.y)
        };

        if ui.button_with_size(&self.label, [button_width, BUTTON_HEIGHT]) {
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
            if ui.button_with_size("Warp", [400., BUTTON_HEIGHT]) {
                self.warp();
            }

            let _tok = ui.push_item_width(-1.);
            if ui.button_with_size("Close", [400., BUTTON_HEIGHT])
                || ui.is_key_released(Key::Escape)
            {
                ui.close_current_popup();
            }
        }
    }
}

const GRACES: &[(&str, u32)] = &[
    ("Abandoned Cave", 31202950),
    ("Abandoned Coffin", 1037512950),
    ("Abductor Virgin", 16002962),
    ("Academy Crystal Cave", 31062950),
    ("Academy Gate", 1035462950),
    ("Academy Gate Town", 1037442950),
    ("Across the Roots", 12032955),
    ("Aeonia Swamp Shore", 1048382950),
    ("Agheel Lake North", 1043372950),
    ("Agheel Lake South", 1044352950),
    ("Ailing Village Outskirts", 1044332952),
    ("Ainsel River Downstream", 12012953),
    ("Ainsel River Main", 12012954),
    ("Ainsel River Sluice Gate", 12012952),
    ("Ainsel River Well Depths", 12012951),
    ("Altar South", 1033402950),
    ("Altus Highway Junction", 1039512950),
    ("Altus Plateau", 1038502952),
    ("Altus Tunnel", 32052950),
    ("Ancestral Woods", 12022956),
    ("Ancient Snow Valley Ruins", 1051562950),
    ("Apostate Derelict", 1047582950),
    ("Aqueduct-Facing Cliffs", 12022957),
    ("Artist's Shack", 1038452950),
    ("Artist's Shack", 1044382950),
    ("Astel, Naturalborn of the Void", 12042950),
    ("Astray from Caelid Highway North", 1048382951),
    ("Audience Pathway", 16002960),
    ("Auriza Side Tomb", 30132950),
    ("Auzira Hero's Grave", 30102950),
    ("Behind Caria Manor", 1036502950),
    ("Behind The Castle", 1043312951),
    ("Bellum Church", 1036492950),
    ("Below the Well", 12022959),
    ("Beside the Crater-Pocked Glade", 1045332950),
    ("Beside the Great Bridge", 13002960),
    ("Beside the Rampart Gaol", 1043312952),
    ("Bestial Sanctum", 1051432950),
    ("Black Knife Catacombs", 30052950),
    ("Boilprawn Shack", 1036432950),
    ("Bower of Bounty", 1040532950),
    ("Bridge of Iniquity", 1039532950),
    ("Bridge of Sacrifice", 1044342950),
    ("Caelem Ruins", 1047402950),
    ("Caelid Catacombs", 30152950),
    ("Caelid Highway South", 1048372950),
    ("Capital Rampart", 1045522950),
    ("Castellan's Hall", 1039542952),
    ("Castle Morne Lift", 1043312950),
    ("Castle Morne Rampart", 1044332950),
    ("Castle Sol Main Gate", 1051572951),
    ("Castle Sol Rooftop", 1051572953),
    ("Castleward Tunnel", 10002952),
    ("Cathedral of Dragon Communion", 1048362950),
    ("Cathedral of Manus Celes", 1035422950),
    ("Cathedral of the Forsaken", 35002950),
    ("Cave of Knowledge", 18002950),
    ("Cave of the Forlorn", 31122950),
    ("Chair-Crypt of Sellia", 1049392951),
    ("Chamber Outside the Plaza", 1051362951),
    ("Chapel of Anticipation (New Game)", 10012020),
    ("Church of Dragon Communion", 1041352950),
    ("Church of Elleh", 1042362950),
    ("Church of Inhibition", 1037492950),
    ("Church of Pilgrimage", 1043342950),
    ("Church of Repose", 1051532950),
    ("Church of Vows", 1037462950),
    ("Church of the Cuckoo", 14002952),
    ("Church of the Eclipse", 1051572952),
    ("Church of the Plague", 1050382950),
    ("Cliffbottom Catacombs", 30062950),
    ("Coastal Cave", 31152950),
    ("Cocoon of the Empyrean", 12052950),
    ("Consecrated Snowfield", 1049542950),
    ("Consecrated Snowfield Catacombs", 30192950),
    ("Converted Tower", 1034432950),
    ("Craftsman's Shack", 1036522950),
    ("Crumbling Beast Grave", 13002953),
    ("Crumbling Beast Grave Depths", 13002954),
    ("Crystalline Woods", 1034462950),
    ("Deathtouched Catacombs", 30112950),
    ("Debate Parlour", 14002951),
    ("Deep Siofra Well", 1048402950),
    ("Deeproot Depths", 12032953),
    ("Divine Bridge", 11052955),
    ("Divine Tower of Caelid", 34132951),
    ("Divine Tower of Caelid", 34132952),
    ("Divine Tower of Limgrave", 34102952),
    ("Divine Tower of Liurnia", 34112952),
    ("Divine Tower of West Altus", 34122950),
    ("Divine Tower of West Altus", 34122952),
    ("Divine Tower of the East Altus", 34142950),
    ("Divine Tower of the East Altus", 34142951),
    ("Dragon Temple", 13002956),
    ("Dragon Temple Altar", 13002952),
    ("Dragon Temple Lift", 13002958),
    ("Dragon Temple Rooftop", 13002959),
    ("Dragon Temple Transept", 13002957),
    ("Dragonbarrow Cave", 31102950),
    ("Dragonbarrow Fork", 1050402950),
    ("Dragonbarrow West", 1048402951),
    ("Dragonkin Soldier of Nokstella", 12012950),
    ("Dragonlord Placidusax", 13002951),
    ("Drainage Channel", 15002953),
    ("Dynasty Mausoleum Entrance", 12052952),
    ("Dynasty Mausoleum Midpoint", 12052953),
    ("Earthbore Cave", 31012950),
    ("East Capital Rampart", 11052952),
    ("East Gate Bridge Trestle", 1035472950),
    ("East Raya Lucaria Gate", 1036482950),
    ("Eastern Liurnia Lake Shore", 1038452951),
    ("Eastern Tableland", 1038462950),
    ("Elden Throne", 11052950),
    ("Elphael Inner Wall", 15002952),
    ("Erdtree Sanctuary", 11052951),
    ("Erdtree-Gazing Hill", 1038512950),
    ("Fallen Ruins of the Lake", 1036432951),
    ("Farum Greatbridge", 1052422950),
    ("Fire Giant", 1053522950),
    ("First Church of Marika", 1054552950),
    ("First Mt. Gelmir Campsite", 1038542950),
    ("Folly on the Lake", 1035432950),
    ("Foot of the Forge", 1052532950),
    ("Foot of the Four Belfries", 1033462950),
    ("Forbidden Lands", 1047512950),
    ("Forest-Spanning Greatbridge", 1040522950),
    ("Forge of the Giants", 1054532950),
    ("Forsaken Depths", 35002952),
    ("Fort Faroth", 1051392950),
    ("Fort Gael North", 1047392950),
    ("Fort Haight West", 1045362950),
    ("Fourth Church of Marika", 1041332950),
    ("Fractured Marika", 19002950),
    ("Freezing Lake", 1052572950),
    ("Frenzied Flame Proscription", 35002954),
    ("Frenzied Flame Village Outskirts", 1038482950),
    ("Gael Tunnel", 32072950),
    ("Gaol Cave", 31212950),
    ("Gate Town Bridge", 1038432950),
    ("Gate Town North", 1036452950),
    ("Gatefront", 1042372950),
    ("Gateside Chamber", 10002953),
    ("Gelmir Hero's Grave", 30092950),
    ("Giant's Gravepost", 1052542950),
    ("Giant's Mountaintop Catacombs", 30182950),
    ("Giant-Conquering Hero's Grave", 30172950),
    ("Grand Cloister", 12012958),
    ("Grand Lift of Dectus", 1038502950),
    ("Grand Lift of Rold", 1049532951),
    ("Great Waterfall Basin", 12022950),
    ("Great Waterfall Crest", 12032952),
    ("Groveside Cave", 31032950),
    ("Guest Hall", 16002954),
    ("Haligtree Canopy", 15002956),
    ("Haligtree Promenade", 15002955),
    ("Haligtree Roots", 15002954),
    ("Haligtree Town", 15002957),
    ("Haligtree Town Plaza", 15002958),
    ("Heart of Aeonia", 1049382950),
    ("Hermit Merchant's Shack", 1043532950),
    ("Hidden Path to the Haligtree", 30202950),
    ("Highroad Cave", 31172950),
    ("Impaler's Catacombs", 30012950),
    ("Impassable Greatbridge", 1050362950),
    ("Inner Aeonia", 1049382951),
    ("Inner Consecrated Snowfield", 1049552950),
    ("Isolated Divine Tower", 34152950),
    ("Isolated Merchant's Shack", 1041322950),
    ("Isolated Merchant's Shack", 1048412950),
    ("Jarburg", 1039442950),
    ("Lake of Rot Shoreside", 12012956),
    ("Lake-Facing Cliffs", 1039402950),
    ("Lakeside Crystal Cave", 31052950),
    ("Laskyar Ruins", 1038412950),
    ("Lenne's Rise", 1052412950),
    ("Leyendell, Capital of Ash", 11052953),
    ("Leyndell Catacombs", 35002953),
    ("Liftside Chamber", 10002956),
    ("Limgrave Tower Bridge", 34102950),
    ("Limgrave Tunnels", 32012950),
    ("Liurnia Highway North", 1039422950),
    ("Liurnia Highway South", 1039412950),
    ("Liurnia Lake Shore", 1038402950),
    ("Liurnia Tower Bridge", 34112951),
    ("Magma Wyrm", 39202950),
    ("Main Caria Manor Gate", 1035502953),
    ("Malenia, Godess of Rot", 15002950),
    ("Maliketh, the Black Blade", 13002950),
    ("Manor Lower Level", 1035502951),
    ("Manor Upper Level", 1035502950),
    ("Margit, the Fell Omen", 10002951),
    ("Mausoleum Compound", 1037482950),
    ("Mimic Tear", 12022951),
    ("Minor Eerdtree Catacombs", 30142950),
    ("Minor Eerdtree Church", 1043502950),
    ("Mistwood Outskirts", 1044372950),
    ("Moonlight Altar", 1034412950),
    ("Morne Moangrave", 1043302950),
    ("Morne Tunnel", 32002950),
    ("Murkwater Catacombs", 30042950),
    ("Murkwater Cave", 31002950),
    ("Murkwater Coast", 1043382950),
    ("Night's Sacred Ground", 12022958),
    ("Ninth Mt. Gelmir Campsite", 1036542951),
    ("Nokron, Eternal City", 12072951),
    ("Nokstella Waterfall Basin", 12012959),
    ("Nokstella, Eternal City", 12012955),
    ("Northern Liurnia Lake Shore", 1034482950),
    ("Old Altus Tunnel", 32042950),
    ("Ordina, Liturgical Town", 1048572950),
    ("Outer Wall Battleground", 1043532951),
    ("Outer Wall Phantom Tree", 1042512950),
    ("Palace Approach Ledge-Road", 12052951),
    ("Perfumer's Grotto", 31182950),
    ("Prayer Room", 15002951),
    ("Primeval Sorcerer Azur", 1037532950),
    ("Prince of Death's Throne", 12032950),
    ("Prison Town Church", 16002953),
    ("Queen's Bedchamber", 11052954),
    ("Rampart Tower", 10002955),
    ("Rampartside Path", 1041522951),
    ("Ranni's Chamber", 1034502951),
    ("Ranni's Rise", 1034502950),
    ("Ravine-Veiled Village", 1038502951),
    ("Raya Lucaria Crystal Tunnel", 32022950),
    ("Raya Lucaria Grand Library", 14002950),
    ("Rear Gael Tunnel Entrance", 32072951),
    ("Redmane Castle Plaza", 1051362950),
    ("Revenger's Shack", 1033442950),
    ("Road of Iniquity", 1036542952),
    ("Road of Iniquity Side Path", 1040542950),
    ("Road to the Manor", 1034492950),
    ("Road's End Catacombs", 30032950),
    ("Root-Facing Cliffs", 12032951),
    ("Rotview Balcony", 1046402951),
    ("Royal Moongazing Grounds", 1035502952),
    ("Ruin-Strewn Precipice", 39202951),
    ("Ruin-Strewn Precipice Overlook", 39202952),
    ("Ruined Labyrinth", 1038472950),
    ("Rykard, Lord of Blasphemy", 16002950),
    ("Sage's Cave", 31192950),
    ("Sainted Hero's Grave", 30082950),
    ("Saintsbridge", 1043392950),
    ("Scenic Isle", 1037422950),
    ("Schoolhouse Classroom", 14002953),
    ("Sealed Tunnel", 34122951),
    ("Seaside Ruins", 1043352950),
    ("Secluded Cell", 10002957),
    ("Seethewater Cave", 31072950),
    ("Seethewater River", 1037522951),
    ("Seethewater Terminus", 1035532950),
    ("Sellia Backstreets", 1049392950),
    ("Sellia Crystal Tunnel", 32082950),
    ("Sellia Hideaway", 31112950),
    ("Sellia Under-Stair", 1049392952),
    ("Shaded Castle Inner Gate", 1039542951),
    ("Shaded Castle Ramparts", 1039542950),
    ("Siofra River Bank", 12022953),
    ("Siofra River Well Depths", 12072950),
    ("Slumbering Wolf Shack", 1036412950),
    ("Smoldering Church", 1046402950),
    ("Smoldering Wall", 1048392950),
    ("Snow Valley Ruins Overlook", 1051572950),
    ("Sorcerer's Isle", 1034472951),
    ("South Raya Lucaria Gate", 1035452950),
    ("South of the Lookout Tower", 1044332951),
    ("Southern Aeonia Swamp Bank", 1049372950),
    ("Spiritcaller's Cave", 31222950),
    ("Starscourge Radahn", 1052382950),
    ("Stillwater Cave", 31042950),
    ("Stormfoot Catacombs", 30022950),
    ("Stormhill Shack", 1041382950),
    ("Stormveil Cliffside", 10002954),
    ("Stormveil Main Gate", 10002958),
    ("Stranded Graveyard", 18002951),
    ("Study Hall Entrance", 34112950),
    ("Subterranean Inquisition Chamber", 16002964),
    ("Summonwater Village Outskirts", 1044392950),
    ("Table of Lost Grace", 11102950),
    ("Tempest-Facing Balcony", 13002955),
    ("Temple Quarter", 1034442950),
    ("Temple of Eiglay", 16002951),
    ("The First Step", 1042362951),
    ("The Four Belfries", 1033472950),
    ("The Nameless Eternal City", 12032954),
    ("The Ravine", 1036492951),
    ("Third Church of Marika", 1046382950),
    ("Tombsward", 1042332950),
    ("Tombsward Catacombs", 30002950),
    ("Tombsward Cave", 31022950),
    ("Underground Roadside", 35002951),
    ("Unsightly Catacombs", 30122950),
    ("Village of the Albinaurics", 1034422950),
    ("Volcano Cave", 31092950),
    ("Volcano Manor", 16002952),
    ("War-Dead Catacombs", 30162950),
    ("Warmaster's Shack", 1042382950),
    ("Waypoint Ruins Cellar", 1044362950),
    ("Whiteridge Road", 1052562950),
    ("Windmill Heights", 1042552950),
    ("Windmill Village", 1041542950),
    ("Worshippers' Woods", 12022954),
    ("Wyndham Catacombs", 30072950),
    ("Yelough Anix Tunnel", 32112950),
    ("Zamor Ruins", 1049532950),
];

// type PackCoordsFunc = extern "system" fn(u32, u32, u32, u32);
// type WarpFunc = extern "system" fn(u32, u32);
//
// #[derive(Debug)]
// pub(crate) struct Warp {
//     label: String,
//     pack_coords_func_ptr: usize,
//     warp_func_ptr: usize,
//     hotkey: KeyState,
// }
//
// impl Warp {
//     pub(crate) fn new(pack_coords_func_ptr: usize, warp_func_ptr: usize,
// hotkey: KeyState) -> Self {         Warp { label: format!("Warp"),
// pack_coords_func_ptr, warp_func_ptr, hotkey }     }
//
//     fn warp(&self) {
//         let pack_coords: WarpFunc = unsafe {
// mem::transmute(self.pack_coords_func_ptr) };         let warp: WarpFunc =
// unsafe { mem::transmute(self.warp_func_ptr) };     }
// }
//
// impl Widget for Warp {
//     fn render(&mut self, ui: &imgui::Ui) {
//         let scale = super::scaling_factor(ui);
//
//         if ui.button_with_size(&self.label, [super::BUTTON_WIDTH * scale,
// super::BUTTON_HEIGHT]) {             self.warp();
//         }
//     }
//
//     fn interact(&mut self, ui: &imgui::Ui) {
//         if self.hotkey.keyup(ui) {
//             self.warp();
//         }
//     }
// }
