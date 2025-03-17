use crate::{selectable::Selectable, ui::LABELS, widgets::popup::Multiselect};
use std::collections::{HashMap, HashSet};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::{Condition, ConditionDuration};
use ratatui::prelude::*;
use super::AfterKey;

/// State for applying conditions to combatants.
#[derive(Clone, Debug, Default)]
pub struct ApplyCondition {
    /// The conditions to apply to combatants.
    conditions: HashSet<Condition>,

    /// Duration of the conditions.
    duration: ConditionDuration,
}

impl ApplyCondition {
    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(Multiselect::new(
            "Select condition(s)",
            &self.conditions,
        ), frame.area());
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: KeyEvent) -> AfterKey {
        // generate labels for all conditions
        let label_to_option = LABELS
            .chars()
            .zip(Condition::variants())
            .collect::<HashMap<_, _>>();

        match key.code {
            KeyCode::Enter => return AfterKey::Exit,
            KeyCode::Char(label) => {
                let selected = &mut self.conditions;
                if let Some(option) = label_to_option.get(&label) {
                    if selected.contains(option) {
                        selected.remove(option);
                    } else {
                        selected.insert(*option);
                    }
                }
            },
            _ => (),
        }

        AfterKey::Stay
    }

    /// Apply the conditions to the tracker.
    pub fn apply(&self, tracker: &mut h5t_core::Tracker) {
        todo!()
    }
}
