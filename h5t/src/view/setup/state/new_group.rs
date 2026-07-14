use crate::{
    input::{AfterKey as AfterKeyInner, Charset, GetInput},
    theme::{Rgb, THEME},
    view::setup::SetupInner,
    widgets::{popup::Popup, ColorPicker},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::Flex, prelude::*};
use super::AfterKey;

/// State for creating a new group combatants can be assigned to.
#[derive(Clone, Debug)]
pub struct NewGroup {
    /// Helper for the group name input.
    group_name: GetInput<String>,

    /// The color of the group.
    color: Rgb,

    /// The index of the RGB color field to modify.
    color_idx: u8,
}

impl NewGroup {
    /// Create a [`NewGroup`] state.
    pub fn new() -> Self {
        Self {
            group_name: GetInput::new("Group name", 20, Charset::All),
            color: Rgb(255, 255, 255),
            color_idx: 0,
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let [group_name, color_picker] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(9),
        ])
            .flex(Flex::Center)
            .areas(area);

        self.group_name.draw(frame, group_name);

        let widget = ColorPicker::new(self.color, 40);
        let popup = Popup::new(THEME.foreground, Some("Choose group color"), true, widget);
        popup.render(color_picker, frame.buffer_mut());
    }

    /// Reduces the selected color channel by the given amount.
    fn less(&mut self, amount: u8) {
        match self.color_idx {
            0 => self.color.0 = self.color.0.wrapping_sub(amount),
            1 => self.color.1 = self.color.1.wrapping_sub(amount),
            2 => self.color.2 = self.color.2.wrapping_sub(amount),
            _ => unreachable!(),
        }
    }

    /// Increases the selected color channel by the given amount.
    fn more(&mut self, amount: u8) {
        match self.color_idx {
            0 => self.color.0 = self.color.0.wrapping_add(amount),
            1 => self.color.1 = self.color.1.wrapping_add(amount),
            2 => self.color.2 = self.color.2.wrapping_add(amount),
            _ => unreachable!(),
        }
    }

    /// Handle a key event and apply any needed changes to the combatant list.
    pub fn handle_key(&mut self, key: KeyEvent, inner: &mut SetupInner) -> AfterKey {
        /// Returns the amount to change a color channel by, given the key modifiers.
        fn change_amount(modifiers: KeyModifiers) -> u8 {
            match (
                modifiers.contains(KeyModifiers::SHIFT),
                modifiers.contains(KeyModifiers::CONTROL),
            ) {
                (false, false) => 1,
                (true, false) => 10,
                (false, true) => 30,
                (true, true) => 60,
            }
        }

        match self.group_name.handle_key(key) {
            AfterKeyInner::Handled => (),
            AfterKeyInner::Submit(group_name) => {
                inner.groups.insert(group_name, self.color);
                return AfterKey::Exit;
            },
            AfterKeyInner::Cancel => return AfterKey::Exit,
            AfterKeyInner::Forward(key) => match key.code {
                KeyCode::Esc => return AfterKey::Exit,
                KeyCode::Left => self.less(change_amount(key.modifiers)),
                KeyCode::Right => self.more(change_amount(key.modifiers)),
                KeyCode::Up => self.color_idx = self.color_idx.checked_sub(1).unwrap_or(2),
                KeyCode::Down => {
                    self.color_idx += 1;
                    if self.color_idx >= 3 {
                        self.color_idx = 0;
                    }
                },
                _ => (),
            },
        }

        AfterKey::Stay
    }
}
