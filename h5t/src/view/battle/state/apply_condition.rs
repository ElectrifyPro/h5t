use crate::{
    input::{AfterKey as AfterKeyInput, Charset, GetInput},
    selectable::SelectableEnum,
    view::LABELS,
    widgets::popup::{popup_area, Multiselect, Select},
    Tracker,
};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::{Condition, ConditionDuration, ConditionKind};
use ratatui::{layout::Flex, prelude::*};
use std::{collections::{HashMap, HashSet}, num::NonZeroU32};
use super::AfterKey;

/// Helper enum to indicate which form field is currently selected.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Field {
    #[default]
    Conditions,
    Duration,
}

/// Helper enum to render condition durations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Unit {
    #[default]
    UntilNextTurn,
    Round,
    Minute,
    Forever,
}

impl SelectableEnum for Unit {
    fn variants() -> &'static [Self] {
        &[
            Unit::UntilNextTurn,
            Unit::Round,
            Unit::Minute,
            Unit::Forever,
        ]
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::UntilNextTurn => write!(f, "Until end of next turn"),
            Unit::Round => write!(f, "Rounds"),
            Unit::Minute => write!(f, "Minutes"),
            Unit::Forever => write!(f, "Forever"),
        }
    }
}

/// State for applying conditions to combatants.
#[derive(Clone, Debug)]
pub struct ApplyCondition {
    /// Indicates which form field is currently selected.
    field: Field,

    /// The combatant indices to apply damage to.
    combatants: HashSet<usize>,

    /// The conditions to apply to combatants.
    conditions: HashSet<ConditionKind>,

    /// Helper to get the condition duration from the user.
    duration: GetInput<NonZeroU32>,

    /// Duration of the conditions.
    unit: Unit,
}

impl ApplyCondition {
    /// Create an [`ApplyCondition`] state with the initial state.
    pub fn new(combatants: HashSet<usize>) -> Self {
        Self {
            combatants,
            conditions: HashSet::new(),
            field: Field::default(),
            duration: GetInput::new("Duration", 4, Charset::Numeric) // number of rounds / minutes is usually 1-2 digits
                .suffix(Unit::default().to_string()),
            unit: Unit::default(),
        }
    }

    /// Apply the conditions to the tracker.
    fn apply(&self, tracker: &mut h5t_core::Tracker, amount: NonZeroU32) {
        for condition in &self.conditions {
            let duration = match self.unit {
                Unit::UntilNextTurn => ConditionDuration::UntilNextTurn,
                Unit::Round => ConditionDuration::Rounds(amount),
                Unit::Minute => ConditionDuration::Minutes(amount),
                Unit::Forever => ConditionDuration::Forever,
            };

            for combatant_idx in &self.combatants {
                let combatant = &mut tracker.combatants[*combatant_idx];

                // if the condition is already present, override its length if the new one is longer
                // otherwise, add the condition
                let existing_condition = combatant
                    .conditions
                    .iter_mut()
                    .find(|c| c.kind == *condition);

                if let Some(existing_condition) = existing_condition {
                    // override its length if the new one is longer
                    if duration > existing_condition.duration {
                        existing_condition.duration = duration;
                    }
                } else {
                    // add new condition
                    combatant.conditions.push(Condition {
                        kind: *condition,
                        duration,
                    });
                }
            }
        }
    }

    /// Handle a key event during the [`Field::Conditions`] step.
    fn conditions_field(&mut self, key: KeyEvent) -> AfterKey {
        match key.code {
            KeyCode::Esc => return AfterKey::Exit,
            KeyCode::Enter => {
                self.field = Field::Duration;
                self.duration.set_active(true);
            },
            KeyCode::Char(label) => {
                let label_to_option = LABELS
                    .into_iter()
                    .zip(ConditionKind::owned_variants())
                    .collect::<HashMap<_, _>>();

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

    /// Handle a key event during the [`Field::Duration`] step.
    fn duration_field(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        // NOTE: the input field handles all input, even if the `Unit::UntilNextTurn` or
        // `Unit::Forever` is selected, just for a simple implementation
        match self.duration.handle_key(key) {
            AfterKeyInput::Handled => (),
            AfterKeyInput::Submit(amount) => {
                self.apply(tracker, amount);
                return AfterKey::Exit;
            },
            AfterKeyInput::Cancel => {
                self.field = Field::Conditions;
                self.duration.set_active(false);
            },
            AfterKeyInput::Forward(key) => {
                let KeyCode::Char(label) = key.code else {
                    return AfterKey::Stay;
                };

                let label_to_option = LABELS
                    .into_iter()
                    .zip(Unit::owned_variants())
                    .collect::<HashMap<_, _>>();

                let selected = &mut self.unit;
                if let Some(option) = label_to_option.get(&label) {
                    *selected = *option;
                    self.duration.reset_value();
                    self.duration.set_suffix(selected.to_string().to_lowercase());
                }
            },
        }

        AfterKey::Stay
    }
}

impl ApplyCondition {
    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let area = popup_area(
            area,
            HorizontalAlignment::Center,
            VerticalAlignment::Bottom,
            (area.width, area.height / 2),
        );
        let [conditions, duration] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
            .flex(Flex::Center)
            .areas(area);
        let [duration_unit, duration_amount] = Layout::vertical([
            Constraint::Length(6),
            Constraint::Length(3),
        ])
            .flex(Flex::Center)
            .areas(duration);
        frame.render_widget(Multiselect::with_enum(
            "Select condition(s)",
            &self.conditions,
            self.field == Field::Conditions,
        ), conditions);

        frame.render_widget(Select::with_enum(
            "For how long?",
            Some(&self.unit),
            self.field == Field::Duration,
        ), duration_unit);
        if self.unit == Unit::Round || self.unit == Unit::Minute {
            self.duration.draw(frame, duration_amount);
        }
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        match self.field {
            Field::Conditions => self.conditions_field(key),
            Field::Duration => self.duration_field(key, tracker),
        }
    }
}
