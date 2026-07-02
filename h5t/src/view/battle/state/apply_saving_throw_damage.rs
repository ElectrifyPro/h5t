use crate::{
    input::{AfterKey as AfterKeyInner, Charset, GetInput},
    selectable::SelectableEnum,
    view::LABELS,
    widgets::popup::Select,
    Tracker,
};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::{ability::AbilityKind, DamageKind};
use ratatui::{layout::Flex, prelude::*};
use std::collections::{HashMap, HashSet};
use super::AfterKey;

/// Helper enum to indicate which form field is currently selected.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Field {
    #[default]
    Ability,
    Damage,
}

/// State for applying damage to combatants after requiring a saving throw.
#[derive(Clone, Debug, Default)]
pub struct ApplySavingThrowDamage {
    /// Indicates which form field is currently selected.
    selected: Field,

    /// The combatant indices to apply damage to.
    combatants: HashSet<usize>,

    /// The ability combatants must use for the saving throw.
    ability: Option<AbilityKind>,

    /// The type of damage to apply. This can be omitted to simply reduce combatant HP without
    /// dealing with vulnerabilities, resistances, or immunities.
    kind: Option<DamageKind>,

    /// Helper to get input from the user.
    input: GetInput<i32>,
}

impl ApplySavingThrowDamage {
    /// Create an [`ApplySavingThrowDamage`] state with the given combatants.
    pub fn new(combatants: HashSet<usize>) -> Self {
        Self {
            selected: Field::default(),
            combatants,
            ability: None,
            kind: None,
            input: GetInput::new("Damage amount", 4, Charset::Numeric) // damage is usually 1-2 digits
                .active(false)
                .suffix("HP"),
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let [select_ability, damage] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
            .flex(Flex::Center)
            .areas(frame.area());
        let [damage_amount, damage_kind] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(15),
        ])
            .flex(Flex::Center)
            .areas(damage);

        frame.render_widget(Select::with_enum(
            "Select save ability",
            self.ability.as_ref(),
            self.selected == Field::Ability,
        ), select_ability);

        self.input.draw(frame, damage_amount);
        frame.render_widget(Select::with_enum(
            "Select damage type",
            self.kind.as_ref(),
            self.selected == Field::Damage,
        ), damage_kind);
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        if self.selected == Field::Ability {
            let label_to_option = LABELS
                .chars()
                .zip(AbilityKind::owned_variants())
                .collect::<HashMap<_, _>>();

            match key.code {
                KeyCode::Esc => return AfterKey::Exit,
                KeyCode::Enter => {
                    self.selected = Field::Damage;
                    self.input.set_active(true);
                },
                KeyCode::Char(label) => {
                    if let Some(&option) = label_to_option.get(&label) {
                        if self.ability == Some(option) {
                            self.ability = None;
                        } else {
                            self.ability = Some(option);
                        }
                    }
                },
                _ => (),
            }

            AfterKey::Stay
        } else {
            match self.input.handle_key(key) {
                AfterKeyInner::Handled => AfterKey::Stay,
                AfterKeyInner::Submit(value) => {
                    for combatant_idx in &self.combatants {
                        let combatant = &mut tracker.combatants[*combatant_idx];
                        combatant.damage(value, self.kind);
                    }
                    AfterKey::Exit
                },
                AfterKeyInner::Cancel => {
                    self.selected = Field::Ability;
                    self.input.set_active(false);
                    AfterKey::Stay
                },
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
}
