use imgui::sys::{igGetCursorPosX, igGetCursorPosY, igGetWindowPos, igSetNextWindowPos, ImVec2};
use imgui::Condition;

use super::{Widget, BUTTON_HEIGHT, BUTTON_WIDTH};
use crate::util::KeyState;

#[derive(Debug)]
pub(crate) struct Group {
    label: String,
    label_close: String,
    tag: String,
    hotkey_close: KeyState,
    commands: Vec<Box<dyn Widget>>,
}

impl Group {
    pub(crate) fn new(label: &str, hotkey_close: KeyState, commands: Vec<Box<dyn Widget>>) -> Self {
        Self {
            label: label.to_string(),
            tag: format!("##group-{label}"),
            label_close: format!("Close ({hotkey_close})"),
            hotkey_close,
            commands,
        }
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

            if ui.button_with_size(&self.label_close, [button_width, BUTTON_HEIGHT])
                || (self.hotkey_close.keyup(ui) && !ui.is_any_item_active())
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

    fn log(&mut self) -> Option<Vec<String>> {
        self.commands.iter_mut().filter_map(|c| c.log()).fold(None, |o, i| {
            let mut o = o.unwrap_or_default();
            o.extend(i);
            Some(o)
        })
    }
}
