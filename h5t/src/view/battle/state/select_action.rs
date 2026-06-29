use crate::{Tracker, theme::THEME, view::LABELS, widgets::{SelectableTable, popup::Popup}};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::{resource::ResourcePool, Action};
use ratatui::prelude::*;
use super::AfterKey;

/// State for choosing an action to spend an action point on.
#[derive(Clone, Debug, Default)]
pub struct SelectAction {
    /// The actions to select from.
    actions: Vec<Action>,

    /// The resources the combatant currently has.
    pool: ResourcePool,

    /// Index of the selected action.
    selected: Option<usize>,
}

impl SelectAction {
    /// Create an [`SelectAction`] state with the given combatants.
    pub fn new(actions: Vec<Action>, pool: ResourcePool) -> Self {
        Self {
            actions,
            pool,
            selected: None,
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let widget = SelectableTable::<Action, _, _>::with_options(
            &self.actions,
            |idx, _: &Action| if let Some(selected_idx) = self.selected {
                selected_idx == idx
            } else {
                false
            },
            |_, action: &Action| self.pool.can_perform(action.costs()),
        );

        let area = frame.area();
        let buf = frame.buffer_mut();
        let popup = Popup::new(THEME.foreground, Some("Select action"), true, widget);
        popup.render(area, buf);
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        match key.code {
            KeyCode::Esc => AfterKey::Exit,
            KeyCode::Enter => if let Some(idx) = self.selected {
                tracker.use_action(&self.actions[idx]);
                AfterKey::Exit
            } else {
                AfterKey::Stay
            },
            KeyCode::Char(label) => {
                let Some(idx) = LABELS.chars().position(|ch| ch == label) else {
                    return AfterKey::Stay;
                };

                if idx < self.actions.len() {
                    self.selected = Some(idx);
                }

                AfterKey::Stay
            },
            _ => AfterKey::Stay,
        }
    }
}
