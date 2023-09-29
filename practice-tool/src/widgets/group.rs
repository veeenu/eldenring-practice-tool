use imgui::sys::{igSetNextWindowPos, ImVec2};
use imgui::{Condition, Key};
use imgui_sys::{igGetCursorPosX, igGetCursorPosY, igGetWindowPos};

use super::{Widget, BUTTON_HEIGHT, BUTTON_WIDTH};

#[derive(Debug)]
pub(crate) struct Group {
    label: String,
    tag: String,
    commands: Vec<Box<dyn Widget>>,
}

impl Group {
    pub(crate) fn new(label: &str, commands: Vec<Box<dyn Widget>>) -> Self {
        Self { label: label.to_string(), tag: format!("##group-{label}"), commands }
    }
}

impl Widget for Group {
    fn render(&mut self, ui: &imgui::Ui) {
        let scale = super::scaling_factor(ui);
        let button_width = BUTTON_WIDTH * scale;

        let (x, y) = unsafe {
            let mut wnd_pos = ImVec2::default();
            igGetWindowPos(&mut wnd_pos);
            (igGetCursorPosX() + wnd_pos.x, igGetCursorPosY() + wnd_pos.y)
        };

        if ui.button_with_size(&self.label, [button_width, BUTTON_HEIGHT]) {
            ui.open_popup(&self.tag);
        }

        unsafe {
            igSetNextWindowPos(
                ImVec2::new(x + 200. * scale, y),
                Condition::Always as i8 as _,
                ImVec2::new(0., 0.),
            )
        };

        if let Some(_token) = ui
            .modal_popup_config(&self.tag)
            .resizable(false)
            .movable(false)
            .title_bar(false)
            .scroll_bar(false)
            .begin_popup()
        {
            for widget in &mut self.commands {
                widget.render(ui);
            }

            if ui.button_with_size("Close", [button_width, BUTTON_HEIGHT])
                || ui.is_key_released(Key::Escape)
            {
                ui.close_current_popup();
            }
        }
    }

    fn render_closed(&mut self, ui: &imgui::Ui) {
        for widget in &mut self.commands {
            widget.render_closed(ui);
        }
    }

    fn interact(&mut self, ui: &imgui::Ui) {
        for widget in &mut self.commands {
            widget.interact(ui);
        }
    }

    fn interact_ui(&mut self, ui: &imgui::Ui) {
        for widget in &mut self.commands {
            widget.interact_ui(ui);
        }
    }

    fn log(&mut self) -> Option<Vec<String>> {
        self.commands.iter_mut().filter_map(|c| c.log()).fold(None, |o, i| {
            let mut o = o.unwrap_or_default();
            o.extend(i);
            Some(o)
        })
    }
}
