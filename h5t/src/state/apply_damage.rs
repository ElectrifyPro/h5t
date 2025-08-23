use crate::{input::{AfterKey as AfterKeyInner, GetInput}, Tracker};
use crossterm::event::KeyEvent;
use ratatui::prelude::*;
use super::AfterKey;

/// State for applying damage to combatants.
#[derive(Clone, Debug, Default)]
pub struct ApplyDamage {
    /// The combatant indices to apply damage to.
    combatants: Vec<usize>,

    /// Helper to get input from the user.
    input: GetInput<i32>,
}

impl ApplyDamage {
    /// Create an [`ApplyDamage`] state with the given combatants.
    pub fn new(combatants: Vec<usize>) -> Self {
        Self {
            combatants,
            input: GetInput::new("Damage amount", 4), // damage is usually 1-2 digits
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        self.input.draw(frame, frame.area());
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        match self.input.handle_key(key) {
            AfterKeyInner::Handled => AfterKey::Stay,
            AfterKeyInner::Submit(value) => {
                for combatant_idx in &self.combatants {
                    let combatant = &mut tracker.combatants[*combatant_idx];
                    combatant.damage(value);
                }
                AfterKey::Exit
            },
            AfterKeyInner::Cancel => AfterKey::Exit,
            _ => AfterKey::Stay,
        }
    }
}
