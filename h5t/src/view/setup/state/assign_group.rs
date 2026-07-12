use crate::{
    theme::THEME,
    view::{LABELS, setup::SetupInner},
    widgets::{SelectableTable, popup::{Popup, Select}, selectable_table::infer},
};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::Combatant;
use ratatui::{layout::Flex, prelude::*};
use std::collections::HashMap;
use super::AfterKey;

/// Helper enum to indicate which form field is currently selected.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Field {
    #[default]
    Groups,
    Combatants,
}

/// Combatant data needed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
struct CombatantData {
    /// The index of the combatant in the setup view.
    idx: usize,

    /// The combatant's name.
    name: String,

    /// The combatant's assigned group.
    group: Option<String>,
}

impl CombatantData {
    /// Create a [`CombatantData`] with a combatant.
    fn new((idx, combatant): (usize, &Combatant)) -> Self {
        Self {
            idx,
            name: combatant.name().to_string(),
            group: combatant.group.clone(),
        }
    }
}

/// State for assigning combatants to groups.
#[derive(Clone, Debug)]
pub struct AssignGroup {
    /// Indicates which form field is currently selected.
    field: Field,

    /// Map of groups to their associated colors.
    group_colors: HashMap<String, Color>,

    /// The group names to choose from.
    group_names: Vec<String>,

    /// The name of the group to assign.
    group_to_assign: String,

    /// The combatants to assign to groups.
    combatants: Vec<CombatantData>,
}

impl AssignGroup {
    /// Create an [`AssignGroup`] state with the given setup state.
    pub fn new(inner: &SetupInner) -> Self {
        Self {
            field: Field::default(),
            group_colors: inner.groups
                .iter()
                .cloned()
                .collect(),
            group_names: inner.groups
                .iter()
                .map(|(name, _)| name.to_string())
                .collect(),
            group_to_assign: String::new(),
            combatants: inner.combatants
                .iter()
                .enumerate()
                .map(CombatantData::new)
                .collect(),
        }
    }

    fn groups_step(&mut self, key: KeyEvent) -> AfterKey {
        match key.code {
            KeyCode::Esc => return AfterKey::Exit,
            KeyCode::Enter => self.field = Field::Combatants,
            KeyCode::Char(label) => {
                let Some(idx) = LABELS.into_iter().position(|ch| ch == label) else {
                    return AfterKey::Stay;
                };

                if let Some(group_name) = self.group_names.get(idx) {
                    self.group_to_assign = group_name.clone();
                    self.field = Field::Combatants;
                }
            },
            _ => (),
        }

        AfterKey::Stay
    }

    fn combatants_step(&mut self, key: KeyEvent, inner: &mut SetupInner) -> AfterKey {
        match key.code {
            KeyCode::Esc => self.field = Field::Groups,
            KeyCode::Enter => {
                for data in self.combatants.iter_mut() {
                    inner.combatants[data.idx].group = data.group.clone();
                }

                // if all combatants have a group set, we can stop here
                if self.combatants.iter().all(|data| data.group.is_some()) {
                    return AfterKey::Exit;
                } else {
                    // go back and choose a new group
                    self.field = Field::Groups;
                    self.group_to_assign = String::new();
                }
            },
            KeyCode::Char(label) => {
                let Some(idx) = LABELS.into_iter().position(|ch| ch == label) else {
                    return AfterKey::Stay;
                };

                let Some(combatant) = self.combatants.get_mut(idx) else {
                    return AfterKey::Exit;
                };

                if let Some(group) = &combatant.group && *group == self.group_to_assign {
                    // reset to previous group (can also be `None`)
                    combatant.group = inner.combatants[combatant.idx].group.clone();
                } else {
                    // assign the group
                    combatant.group = Some(self.group_to_assign.clone());
                }
            },
            _ => (),
        }

        AfterKey::Stay
    }
}

impl AssignGroup {
    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let [groups, combatants] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
            .flex(Flex::Center)
            .areas(area);

        frame.render_widget(Select::with_options(
            "Select group",
            &self.group_names,
            Some(&self.group_to_assign),
            self.field == Field::Groups,
        ), groups);

        let selected_combatants = self.combatants
            .iter()
            .filter(|data| data.group.as_ref() == Some(&self.group_to_assign))
            .cloned()
            .collect::<Vec<_>>();

        let widget = SelectableTable::with_options(
            &self.combatants,
            |_: usize, combatant: &CombatantData| selected_combatants.contains(combatant),
            |_: usize, _: &CombatantData| true,
            infer(|_: usize, combatant: &CombatantData| Text::styled(
                &combatant.name,
                if let Some((_, color)) = self.group_colors.iter().find(|data| Some(data.0) == combatant.group.as_ref()) {
                    *color
                } else {
                    THEME.foreground.into()
                },
            ))
        );
        let prompt = format!("Assign combatants ({}/{})", selected_combatants.len(), self.combatants.len());
        let popup = Popup::new(THEME.foreground, Some(&prompt), self.field == Field::Combatants, widget);
        popup.render(combatants, frame.buffer_mut());
    }

    /// Handle a key event and apply any needed changes to the combatant list.
    pub fn handle_key(&mut self, key: KeyEvent, inner: &mut SetupInner) -> AfterKey {
        match self.field {
            Field::Groups => self.groups_step(key),
            Field::Combatants => self.combatants_step(key, inner),
        }
    }
}
