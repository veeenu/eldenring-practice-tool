use imgui::{ProgressBar, StyleColor};
use libeldenring::memedit::PointerChain;
use libeldenring::pointer_chain;
use practice_tool_core::key::Key;
use practice_tool_core::widgets::Widget;
use windows::Win32::System::Memory::{
    VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
};

#[derive(Debug, Default)]
struct EnemyInfo {
    hp: u32,
    max_hp: u32,
    mp: u32,
    max_mp: u32,
    sp: u32,
    max_sp: u32,
    res: EnemyResistances,
    poise: PoiseMeter,
}

#[derive(Debug, Default)]
#[repr(C)]
struct EnemyResistances {
    poison: u32,
    rot: u32,
    bleed: u32,
    blight: u32,
    frost: u32,
    sleep: u32,
    mad: u32,
    poison_max: u32,
    rot_max: u32,
    bleed_max: u32,
    blight_max: u32,
    frost_max: u32,
    sleep_max: u32,
    mad_max: u32,
}

#[derive(Debug, Default)]
#[repr(C)]
struct PoiseMeter {
    poise: f32,
    poise_max: f32,
    _unk: f32,
    poise_time: f32,
}

struct EntityPointerChains {
    hp: PointerChain<[u32; 3]>,
    sp: PointerChain<[u32; 3]>,
    mp: PointerChain<[u32; 3]>,
    res: PointerChain<EnemyResistances>,
    poise: PointerChain<PoiseMeter>,
}

#[derive(Debug)]
pub(crate) struct Target {
    label: String,
    alloc_addr: PointerChain<[u8; 22]>,
    detour_addr: PointerChain<[u8; 11]>,
    detour_orig_data: [u8; 11],
    hotkey: Key,
    is_enabled: bool,
    entity_addr: u64,
}

unsafe impl Send for Target {}
unsafe impl Sync for Target {}

impl Target {
    pub(crate) fn new(detour_addr: PointerChain<u64>, hotkey: Key) -> Self {
        let detour_addr = detour_addr.cast();
        let mut allocate_near = detour_addr.eval().unwrap() as usize;

        let alloc_addr = loop {
            let c = unsafe {
                VirtualAlloc(
                    Some(allocate_near as *mut _),
                    0x20,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_EXECUTE_READWRITE,
                )
            };
            if c.is_null() {
                allocate_near += 65536;
            } else {
                break pointer_chain!(c as usize);
            }
        };

        Target {
            label: format!("Target entity info ({})", hotkey),
            alloc_addr,
            detour_addr,
            detour_orig_data: Default::default(),
            hotkey,
            is_enabled: false,
            entity_addr: 0,
        }
    }

    fn get_data(&self) -> Option<EnemyInfo> {
        if !self.is_enabled || self.entity_addr == 0 {
            return None;
        }

        let epc = EntityPointerChains {
            hp: pointer_chain!(self.entity_addr as usize + 0x190, 0, 0x138),
            sp: pointer_chain!(self.entity_addr as usize + 0x190, 0, 0x154),
            mp: pointer_chain!(self.entity_addr as usize + 0x190, 0, 0x148),
            res: pointer_chain!(self.entity_addr as usize + 0x190, 0x20, 0x10),
            poise: pointer_chain!(self.entity_addr as usize + 0x190, 0x40, 0x10),
        };

        let [hp, _, max_hp] = epc.hp.read()?;
        let [sp, _, max_sp] = epc.sp.read()?;
        let [mp, _, max_mp] = epc.mp.read()?;
        let res = epc.res.read()?;
        let poise = epc.poise.read()?;

        Some(EnemyInfo { hp, max_hp, mp, max_mp, sp, max_sp, res, poise })
    }

    fn enable(&mut self) {
        // Unwraps are valid because the addresses are static.

        self.detour_orig_data = self.detour_addr.read().unwrap();

        let detour_addr = self.detour_addr.eval().unwrap();
        let alloc_addr = self.alloc_addr.eval().unwrap();

        let data_ptr = (&self.entity_addr as *const u64) as usize;
        let going_jmp_to = (alloc_addr as isize - detour_addr as isize - 5) as i32;
        let returning_jmp_to = (detour_addr as isize - alloc_addr as isize - 11) as i32;

        // jmp going; nop...
        let mut detour_bytes: [u8; 11] = [0xE9, 0, 0, 0, 0, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90];

        let mut patch_data: [u8; 22] = [
            0x48, 0xa3, 0, 0, 0, 0, 0, 0, 0, 0, // mov [data_ptr], rax
            0x48, 0x89, 0x8d, 0, 0, 0, 0, // mov [r13+...], rcx
            0xe9, 0, 0, 0, 0, // jmp returning
        ];

        detour_bytes[1..5].copy_from_slice(&u32_to_array(going_jmp_to as _));
        patch_data[2..10].copy_from_slice(&u64_to_array(data_ptr as _));
        patch_data[10..17].copy_from_slice(&self.detour_orig_data[4..]);
        patch_data[18..].copy_from_slice(&u32_to_array(returning_jmp_to as _));

        self.alloc_addr.write(patch_data);
        self.detour_addr.write(detour_bytes);
        self.is_enabled = true;
    }

    fn disable(&mut self) {
        self.detour_addr.write(self.detour_orig_data);
        self.is_enabled = false;
    }
}

#[inline]
fn u32_to_array(val: u32) -> [u8; 4] {
    let mut buf = [0u8; 4];

    for (i, item) in buf.iter_mut().enumerate() {
        *item = ((val >> (i * 8)) & 0xff) as u8;
    }

    buf
}

#[inline]
fn u64_to_array(val: u64) -> [u8; 8] {
    let mut buf = [0u8; 8];

    for (i, item) in buf.iter_mut().enumerate() {
        *item = ((val >> (i * 8)) & 0xff) as u8;
    }

    buf
}

impl Widget for Target {
    fn render(&mut self, ui: &imgui::Ui) {
        let mut state = self.is_enabled;

        if ui.checkbox(&self.label, &mut state) {
            if state {
                self.enable();
            } else {
                self.disable();
            }
        }
    }

    fn render_closed(&mut self, ui: &imgui::Ui) {
        if !self.is_enabled {
            return;
        }

        let Some(EnemyInfo { hp, max_hp, mp, max_mp, sp, max_sp, res, poise }) = self.get_data()
        else {
            if self.is_enabled {
                ui.text("No enemy locked on")
            };
            return;
        };

        let PoiseMeter { poise, poise_max, _unk, poise_time } = poise;

        let EnemyResistances {
            poison,
            rot,
            bleed,
            blight,
            frost,
            sleep,
            mad,
            poison_max,
            rot_max,
            bleed_max,
            blight_max,
            frost_max,
            sleep_max,
            mad_max,
        } = res;

        #[inline]
        fn div(a: u32, b: u32) -> f32 {
            let a = a as f32;
            let b = b as f32;

            let d = a / b;
            if d.is_nan() {
                0.
            } else {
                d
            }
        }

        let pbar_size: [f32; 2] = [200., 4.];

        // Waiting for const fn float arithmetic... https://github.com/rust-lang/rust/issues/57241
        // const fn conv_color(rgba: u32) -> [f32; 4] {
        //     let r = ((rgba >> 24) & 0xff) as u8;
        //     let g = ((rgba >> 16) & 0xff) as u8;
        //     let b = ((rgba >> 8) & 0xff) as u8;
        //     let a = (rgba & 0xff) as u8;
        //     [(r as f32 / 255.), (g as f32 / 255.), (b as f32 / 255.), (a as f32 /
        // 255.)] }

        const COLOR_BASE: [f32; 4] = [1.0, 0.7529412, 0.4392157, 1.0];
        const COLOR_HP: [f32; 4] = [0.60784316, 0.28627452, 0.28627452, 1.0];
        const COLOR_SP: [f32; 4] = [0.41960785, 0.41960785, 0.8745098, 1.0];
        const COLOR_MP: [f32; 4] = [0.2784314, 0.2784314, 0.5764706, 1.0];
        const COLOR_POISON: [f32; 4] = [0.5137255, 0.19215687, 0.972549, 1.0];
        const COLOR_ROT: [f32; 4] = [0.24313725, 0.035294117, 0.525490196, 1.0];
        const COLOR_BLEED: [f32; 4] = [0.9647059, 0.003921569, 0.23137255, 1.0];
        const COLOR_BLIGHT: [f32; 4] = [0.6823529, 0.6745098, 0.5372549, 1.0];
        const COLOR_FROST: [f32; 4] = [0.627451, 0.70980394, 0.7764706, 1.0];
        const COLOR_SLEEP: [f32; 4] = [0.62745098, 0.70980392, 0.776470588, 1.0];
        const COLOR_MAD: [f32; 4] = [0.62745098, 0.70980392, 0.776470588, 1.0];

        let pbar = |label, cur, max, c| {
            ui.text(format!("{label:8} {cur:>6}/{max:>6}"));
            let pct = div(cur, max);
            let _tok = ui.push_style_color(StyleColor::PlotHistogram, c);
            ProgressBar::new(pct).size(pbar_size).overlay_text("").build(ui);
        };

        pbar("HP", hp, max_hp, COLOR_HP);
        pbar("SP", sp, max_sp, COLOR_SP);
        pbar("MP", mp, max_mp, COLOR_MP);

        ui.text(format!("Poise    {:>6.0}/{:>6.0} {:.2}s", poise, poise_max, poise_time));
        let pct = if poise_max.abs() < 0.0001 { 0.0 } else { poise / poise_max };
        let tok = ui.push_style_color(StyleColor::PlotHistogram, COLOR_BASE);
        ProgressBar::new(pct).size(pbar_size).overlay_text("").build(ui);
        drop(tok);

        pbar("Poison", poison, poison_max, COLOR_POISON);
        pbar("Rot", rot, rot_max, COLOR_ROT);
        pbar("Bleed", bleed, bleed_max, COLOR_BLEED);
        pbar("Blight", blight, blight_max, COLOR_BLIGHT);
        pbar("Frost", frost, frost_max, COLOR_FROST);
        pbar("Sleep", sleep, sleep_max, COLOR_SLEEP);
        pbar("Mad", mad, mad_max, COLOR_MAD);
    }

    fn interact(&mut self, ui: &imgui::Ui) {
        if ui.is_any_item_active() {
            return;
        }

        if self.hotkey.keyup(ui) {
            if self.is_enabled {
                self.disable();
            } else {
                self.enable();
            }
        }
    }
}
