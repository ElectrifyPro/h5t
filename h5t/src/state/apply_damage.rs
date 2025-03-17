use crate::widgets::popup::Input as InputWidget;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use super::AfterKey;

/// State for applying damage to combatants.
#[derive(Clone, Debug, Default)]
pub struct ApplyDamage {
    /// The combatant indices to apply damage to.
    pub combatants: Vec<usize>,

    /// Color of the input field, which changes based on if the input is a valid number.
    color: Color,

    /// The value of the input field.
    pub value: String,
}

impl ApplyDamage {
    /// Create an [`ApplyDamage`] state with the given combatants.
    pub fn new(combatants: Vec<usize>) -> Self {
        Self {
            combatants,
            color: Color::Reset,
            value: String::new(),
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(InputWidget::new(
            self.color,
            "Damage amount",
            &self.value,
            4, // damage is usually 1-2 digits
        ), frame.area());
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: KeyEvent) -> AfterKey {
        match key.code {
            KeyCode::Enter => return AfterKey::Exit,
            KeyCode::Char(c) => self.value.push(c),
            KeyCode::Backspace => { self.value.pop(); },
            _ => (),
        }

        let valid = self.value.trim().parse::<i32>().is_ok();
        self.color = if valid { Color::Reset } else { Color::Red };

        AfterKey::Stay
    }
}
