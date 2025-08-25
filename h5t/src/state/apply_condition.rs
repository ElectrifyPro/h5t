use crate::{
    input::{AfterKey as AfterKeyInput, Charset, GetInput},
    selectable::Selectable,
    ui::LABELS,
    widgets::popup::{popup_area, Multiselect, Select},
    Tracker,
};
use std::{collections::{HashMap, HashSet}, num::NonZeroU32};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::{Condition, ConditionDuration, ConditionKind};
use ratatui::{layout::Flex, prelude::*};
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

impl Selectable for Unit {
    const N: usize = 4;

    fn variants() -> impl Iterator<Item = Self> {
        [
            Unit::UntilNextTurn,
            Unit::Round,
            Unit::Minute,
            Unit::Forever,
        ].into_iter()
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
    /// The conditions to apply to combatants.
    conditions: HashSet<ConditionKind>,

    /// Indicates which form field is currently selected.
    selected: Field,

    /// Helper to get the condition duration from the user.
    input: GetInput<u32>,

    /// Duration of the conditions.
    unit: Unit,
}

impl Default for ApplyCondition {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplyCondition {
    /// Create an [`ApplyCondition`] state with the initial state.
    pub fn new() -> Self {
        Self {
            conditions: HashSet::new(),
            selected: Field::default(),
            input: GetInput::new("Duration", 4, Charset::Numeric) // number of rounds / minutes is usually 1-2 digits
                .suffix(Unit::default().to_string()),
            unit: Unit::default(),
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let area = popup_area(area, Flex::Center, Flex::End, (area.width, area.height / 2), 0);
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
        frame.render_widget(Multiselect::new(
            "Select condition(s)",
            &self.conditions,
            self.selected == Field::Conditions,
        ), conditions);

        if self.unit == Unit::UntilNextTurn || self.unit == Unit::Forever {
            frame.render_widget(Select::new(
                "For how long?",
                &self.unit,
                self.selected == Field::Duration,
            ), duration);
        } else {
            frame.render_widget(Select::new(
                "For how long?",
                &self.unit,
                self.selected == Field::Duration,
            ), duration_unit);
            self.input.draw(frame, duration_amount);
        }
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        // generate labels for all conditions
        if self.selected == Field::Conditions {
            let label_to_option = LABELS
                .chars()
                .zip(ConditionKind::variants())
                .collect::<HashMap<_, _>>();

            match key.code {
                KeyCode::Esc => return AfterKey::Exit,
                KeyCode::Enter => {
                    self.selected = Field::Duration;
                    self.input.set_active(true);
                    return AfterKey::Stay;
                },
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
        } else {
            let label_to_option = LABELS
                .chars()
                .zip(Unit::variants())
                .collect::<HashMap<_, _>>();

            match self.input.handle_key(key) {
                AfterKeyInput::Handled => return AfterKey::Stay,
                AfterKeyInput::Submit(amount) => {
                    self.apply(tracker, amount);
                    return AfterKey::Exit;
                },
                AfterKeyInput::Cancel => {
                    self.selected = Field::Conditions;
                    self.input.set_active(false);
                    return AfterKey::Stay;
                },
                AfterKeyInput::Forward(key) => {
                    let KeyCode::Char(label) = key.code else {
                        return AfterKey::Stay;
                    };

                    let selected = &mut self.unit;
                    if let Some(option) = label_to_option.get(&label) {
                        *selected = *option;
                        self.input.set_suffix(selected.to_string().to_lowercase());
                    }
                },
            }
        }

        AfterKey::Stay
    }

    /// Apply the conditions to the tracker.
    fn apply(&self, tracker: &mut h5t_core::Tracker, amount: u32) {
        for condition in &self.conditions {
            let duration = match self.unit {
                Unit::UntilNextTurn => ConditionDuration::UntilNextTurn,
                Unit::Round => ConditionDuration::Rounds(NonZeroU32::new(amount).unwrap()),
                Unit::Minute => ConditionDuration::Minutes(NonZeroU32::new(amount).unwrap()),
                Unit::Forever => ConditionDuration::Forever,
            };

            // if the condition is already present, override its length if the new one is longer
            // otherwise, add the condition
            let existing_condition = tracker
                .current_combatant_mut()
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
                tracker.current_combatant_mut().conditions.push(Condition {
                    kind: *condition,
                    duration,
                });
            }
        }
    }
}
