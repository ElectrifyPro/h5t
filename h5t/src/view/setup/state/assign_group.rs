use crate::{
    theme::{Rgb, THEME},
    view::{LABELS, setup::SetupInner},
    widgets::{SelectableTable, popup::Popup, selectable_table::infer},
};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::Combatant;
use ratatui::{layout::Flex, prelude::*};
use std::collections::HashMap;
use super::AfterKey;

/// Helper enum to indicate which form field is currently selected.
#[derive(Clone, Debug)]
enum Field {
    /// Select a group to assign.
    Groups {
        /// The currently selected group.
        selected_group: Option<String>,
    },

    /// Assign the previously selected group to combatants.
    Combatants {
        /// The group to assign.
        group_to_assign: String,
    },
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
    group_colors: HashMap<String, Rgb>,

    /// The group names to choose from.
    group_names: Vec<String>,

    /// The combatants to assign to groups.
    combatants: Vec<CombatantData>,
}

impl AssignGroup {
    /// Create an [`AssignGroup`] state with the given setup state.
    pub fn new(inner: &SetupInner) -> Self {
        Self {
            field: Field::Groups { selected_group: None },
            group_colors: inner.groups.clone(),
            group_names: inner.groups
                .iter()
                .map(|(name, _)| name.to_string())
                .collect(),
            combatants: inner.combatants
                .iter()
                .enumerate()
                .map(CombatantData::new)
                .collect(),
        }
    }

    fn groups_step(&mut self, key: KeyEvent) -> AfterKey {
        let Field::Groups { selected_group } = &mut self.field else {
            unreachable!();
        };

        match key.code {
            KeyCode::Esc => return AfterKey::Exit,
            KeyCode::Enter => if let Some(group_to_assign) = selected_group.take() {
                self.field = Field::Combatants { group_to_assign };
            },
            KeyCode::Char(label) => {
                let Some(idx) = LABELS.into_iter().position(|ch| ch == label) else {
                    return AfterKey::Stay;
                };

                if let Some(group) = self.group_names.get(idx) {
                    if let Some(selected_group) = selected_group && selected_group == group {
                        // user selected same group as currently selected group; toggle it off
                        self.field = Field::Groups { selected_group: None };
                    } else {
                        // shortcut: activate `Combatants` right away if no group was selected
                        self.field = Field::Combatants { group_to_assign: group.clone() };
                    }
                }
            },
            _ => (),
        }

        AfterKey::Stay
    }

    fn combatants_step(&mut self, key: KeyEvent, inner: &mut SetupInner) -> AfterKey {
        let Field::Combatants { group_to_assign } = &mut self.field else {
            unreachable!();
        };

        match key.code {
            KeyCode::Esc => self.field = Field::Groups {
                selected_group: Some(std::mem::take(group_to_assign)),
            },
            KeyCode::Enter => {
                for data in self.combatants.iter_mut() {
                    inner.combatants[data.idx].group = data.group.clone();
                }

                // if all combatants have a group set, we can stop here
                if self.combatants.iter().all(|data| data.group.is_some()) {
                    return AfterKey::Exit;
                } else {
                    // go back and choose a new group
                    self.field = Field::Groups { selected_group: None };
                }
            },
            KeyCode::Char(label) => {
                let Some(idx) = LABELS.into_iter().position(|ch| ch == label) else {
                    return AfterKey::Stay;
                };

                let Some(combatant) = self.combatants.get_mut(idx) else {
                    return AfterKey::Stay;
                };

                if let Some(group) = &combatant.group && group == group_to_assign {
                    let previous_group = &inner.combatants[combatant.idx].group;
                    if previous_group.as_ref() == Some(group) {
                        // for when user sets a group, exits this state, then comes back to remove
                        // the group from the combatant
                        combatant.group = None;
                    } else {
                        // reset to group prior to this state (can also be `None`)
                        combatant.group = previous_group.clone();
                    }
                } else {
                    // assign the group
                    combatant.group = Some(group_to_assign.clone());
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

        let group = match &self.field {
            Field::Groups { selected_group } => selected_group.as_ref(),
            Field::Combatants { group_to_assign } => Some(group_to_assign),
        };

        let widget = SelectableTable::with_options(
            &self.group_names,
            |_, name: &String| group == Some(name),
            |_, _: &String| matches!(self.field, Field::Groups { .. }),
            infer(|_, name| Text::styled(
                name,
                {
                    let main_color = self.group_colors.get(name)
                        .copied()
                        .unwrap_or(THEME.foreground);
                    if matches!(self.field, Field::Groups { .. }) {
                        main_color
                    } else {
                        main_color.mix(THEME.background)
                    }
                },
            ))
        );
        let popup = Popup::new(
            THEME.foreground,
            Some("Select group"),
            matches!(self.field, Field::Groups { .. }),
            widget,
        );
        popup.render(groups, frame.buffer_mut());

        let selected_combatants = self.combatants
            .iter()
            .filter(|data| match (data.group.as_ref(), group) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            })
            .collect::<Vec<_>>();

        let widget = SelectableTable::with_options(
            &self.combatants,
            |_, combatant: &CombatantData| selected_combatants.contains(&combatant),
            |_, _: &CombatantData| matches!(self.field, Field::Combatants { .. }),
            infer(|_, combatant: &CombatantData| Text::styled(
                &combatant.name,
                {
                    let main_color = combatant.group
                        .as_ref()
                        .and_then(|group| self.group_colors.get(group))
                        .copied()
                        .unwrap_or(THEME.foreground);
                    if matches!(self.field, Field::Combatants { .. }) {
                        main_color
                    } else {
                        main_color.mix(THEME.background)
                    }
                },
            ))
        );
        let prompt = format!(
            "Assign combatants ({}/{})",
            selected_combatants.len(),
            self.combatants.len(),
        );
        let popup = Popup::new(
            THEME.foreground,
            Some(&prompt),
            matches!(self.field, Field::Combatants { .. }),
            widget,
        );
        popup.render(combatants, frame.buffer_mut());
    }

    /// Handle a key event and apply any needed changes to the combatant list.
    pub fn handle_key(&mut self, key: KeyEvent, inner: &mut SetupInner) -> AfterKey {
        match self.field {
            Field::Groups { .. } => self.groups_step(key),
            Field::Combatants { .. } => self.combatants_step(key, inner),
        }
    }
}
