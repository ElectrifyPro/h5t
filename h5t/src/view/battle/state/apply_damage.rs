use crate::{
    input::{AfterKey as AfterKeyInner, Charset, GetInput},
    selectable::SelectableEnum,
    view::LABELS,
    widgets::popup::Select,
    Tracker,
};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::DamageKind;
use ratatui::{layout::Flex, prelude::*};
use std::collections::{HashMap, HashSet};
use super::AfterKey;

/// State for applying damage to combatants.
#[derive(Clone, Debug, Default)]
pub struct ApplyDamage {
    /// The combatant indices to apply damage to.
    combatants: HashSet<usize>,

    /// The type of damage to apply. This can be omitted to simply reduce combatant HP without
    /// dealing with vulnerabilities, resistances, or immunities.
    kind: Option<DamageKind>,

    /// Helper to get input from the user.
    input: GetInput<i32>,
}

impl ApplyDamage {
    /// Create an [`ApplyDamage`] state with the given combatants.
    pub fn new(combatants: HashSet<usize>) -> Self {
        Self {
            combatants,
            kind: None,
            input: GetInput::new("Damage amount", 4, Charset::Numeric) // damage is usually 1-2 digits
                .suffix("HP"),
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let [damage_amount, damage_kind] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(15),
            ])
            .flex(Flex::Center)
            .areas(frame.area());

        self.input.draw(frame, damage_amount);
        frame.render_widget(Select::with_enum(
            "Select damage type",
            self.kind.as_ref(),
            true,
        ), damage_kind);
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        match self.input.handle_key(key) {
            AfterKeyInner::Handled => AfterKey::Stay,
            AfterKeyInner::Submit(value) => {
                for combatant_idx in &self.combatants {
                    let combatant = &mut tracker.combatants[*combatant_idx];
                    combatant.damage(value, self.kind);
                }
                AfterKey::Exit
            },
            AfterKeyInner::Cancel => AfterKey::Exit,
            AfterKeyInner::Forward(key) => {
                let KeyCode::Char(label) = key.code else {
                    return AfterKey::Stay;
                };

                let label_to_option = LABELS
                    .chars()
                    .zip(DamageKind::owned_variants())
                    .collect::<HashMap<_, _>>();

                if let Some(&option) = label_to_option.get(&label) {
                    if self.kind == Some(option) {
                        self.kind = None;
                    } else {
                        self.kind = Some(option);
                    }
                }

                AfterKey::Stay
            },
        }
    }
}
