use crate::{
    input::{AfterKey as AfterKeyInner, Charset, GetInput},
    selectable::SelectableEnum,
    theme::THEME,
    view::LABELS,
    widgets::{popup::{Popup, Select}, SizedTable},
    Tracker,
};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::{
    ability::{AbilityKind, Modifier, Proficiencies, save_modifier},
    Ability,
    Combatant,
    DamageKind,
};
use ratatui::{layout::Flex, prelude::*, widgets::*};
use std::collections::HashMap;
use super::AfterKey;

/// Formats a dice expression in the form `d20 +/- <modifier>`.
fn fmt_dice_expr(modifier: i32) -> String {
    let operator = match modifier.signum() {
        ..=-1 => '-',
        0 => return "d20 =".to_string(),
        1.. => '+',
    };
    let absolute_value = modifier.abs();
    format!("d20 {} {} =", operator, absolute_value)
}

/// Helper enum to indicate which form field is currently selected.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Field {
    #[default]
    Ability,
    RollSave,
    Damage,
}

/// Combatant data needed.
#[derive(Clone, Debug, Default)]
pub struct CombatantData {
    /// The index of the combatant in the initiative tracker.
    idx: usize,

    /// The combatant's name.
    name: String,

    /// The combatant's ability modifiers.
    modifiers: Ability<Modifier>,

    /// The combatant's saving throw proficiencies.
    proficiencies: Proficiencies,

    /// The combatant's saving throw, with modifiers.
    save: Option<i32>,
}

impl CombatantData {
    /// Create a [`CombatantData`] with a combatant.
    pub fn new(idx: usize, combatant: &Combatant) -> Self {
        Self {
            idx,
            name: combatant.name().to_string(),
            modifiers: combatant.scores().modifiers(),
            proficiencies: combatant.proficiencies(),
            save: None,
        }
    }
}

/// State for applying damage to combatants after requiring a saving throw.
#[derive(Clone, Debug, Default)]
pub struct ApplySavingThrowDamage {
    /// Indicates which form field is currently selected.
    field: Field,

    /// The combatant indices to apply damage to.
    combatants: Vec<CombatantData>,

    /// The combatant within the list of targeted combatants (`self.combatants`) we are rolling a
    /// saving throw for.
    selected_idx: usize,

    /// Helper to get the saving throw from the user.
    saving_throw: GetInput<i32>,

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
    pub fn new(combatants: Vec<CombatantData>) -> Self {
        Self {
            field: Field::default(),
            combatants,
            selected_idx: 0,
            saving_throw: GetInput::new("Saving throw", 3, Charset::Numeric)
                .active(false),
            ability: None,
            kind: None,
            input: GetInput::new("Damage amount", 4, Charset::Numeric) // damage is usually 1-2 digits
                .active(false)
                .suffix("HP"),
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let [select_ability, saving_throws, damage] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
            .flex(Flex::Center)
            .areas(frame.area());
        let [save_input, save_table] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3 + self.combatants.len() as u16),
        ])
            .flex(Flex::Center)
            .areas(saving_throws);
        let [damage_amount, damage_kind] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(15),
        ])
            .flex(Flex::Center)
            .areas(damage);

        frame.render_widget(Select::with_enum(
            "Select save ability",
            self.ability.as_ref(),
            self.field == Field::Ability,
        ), select_ability);

        let roll_save_theme = if self.field == Field::RollSave {
            THEME
        } else {
            THEME.dim()
        };

        self.saving_throw.draw(frame, save_input);

        let longest_combatant_name = self.combatants.iter()
            .map(|data| data.name.len())
            .max();
        let table = SizedTable::new(
            LABELS.into_iter()
                .zip(&self.combatants)
                .enumerate()
                .map(|(idx, (label, data))| {
                    let mut style = Style::default()
                        .fg(roll_save_theme.foreground.into());

                    if self.selected_idx == idx {
                        style = style.bg(roll_save_theme.select.into());
                    }

                    let save_mod = self.ability
                        .map(|ability| save_modifier(ability, data.proficiencies, data.modifiers))
                        .unwrap_or(0);
                    Row::new([
                        Text::from(format!("{}", label)).bold(),
                        Text::raw(&data.name),
                        Text::raw(fmt_dice_expr(save_mod)),
                        Text::raw(data.save.map(|save| save.to_string()).unwrap_or(String::new())),
                    ])
                        .style(style)
                }),
            [
                2,
                2 + longest_combatant_name.unwrap_or(0).max("Combatant".len()) as u16,
                2 + "Expression".len() as u16,
                2 + "Roll result".len() as u16,
            ],
            0,
        )
            .header(
                Row::new([
                    " ",
                    "Combatant",
                    "Expression",
                    "Roll result",
                ])
                    .style(roll_save_theme.foreground)
                    .bold(),
            );
        let popup = Popup::new(
            THEME.foreground,
            Some("Roll saving throws"),
            self.field == Field::RollSave,
            table,
        );
        popup.render(save_table, frame.buffer_mut());

        self.input.draw(frame, damage_amount);
        frame.render_widget(Select::with_enum(
            "Select damage type",
            self.kind.as_ref(),
            self.field == Field::Damage,
        ), damage_kind);
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        if self.field == Field::Ability {
            let label_to_option = LABELS
                .into_iter()
                .zip(AbilityKind::owned_variants())
                .collect::<HashMap<_, _>>();

            match key.code {
                KeyCode::Esc => return AfterKey::Exit,
                KeyCode::Enter => {
                    self.field = Field::RollSave;
                    self.saving_throw.set_active(true);
                },
                KeyCode::Char(label) => {
                    if let Some(&option) = label_to_option.get(&label) {
                        if self.ability == Some(option) {
                            self.ability = None;
                            self.saving_throw.set_prefix("");
                        } else {
                            self.ability = Some(option);

                            let data = &self.combatants[self.selected_idx];
                            let save_mod = self.ability
                                .map(|ability| save_modifier(ability, data.proficiencies, data.modifiers))
                                .unwrap_or(0);
                            self.saving_throw.set_prefix(fmt_dice_expr(save_mod));
                        }
                    }
                },
                _ => (),
            }

            AfterKey::Stay
        } else if self.field == Field::RollSave {
            match self.saving_throw.handle_key(key) {
                AfterKeyInner::Handled => {
                    let Ok(save) = self.saving_throw.get_parsed() else {
                        return AfterKey::Stay;
                    };

                    let combatant = &mut self.combatants[self.selected_idx];
                    combatant.save = Some(save);
                },
                AfterKeyInner::Submit(save) => {
                    let combatant = &mut self.combatants[self.selected_idx];
                    combatant.save = Some(save);

                    // if all combatants have a saving throw, move on to the damage field
                    if self.combatants.iter().all(|data| data.save.is_some()) {
                        self.field = Field::Damage;
                        self.saving_throw.set_active(false);
                        self.input.set_active(true);
                    } else {
                        // move to next combatant with no saving throw set
                        let mut next_idx = self.selected_idx;
                        loop {
                            next_idx = (next_idx + 1) % self.combatants.len();
                            if self.combatants[next_idx].save.is_none() {
                                self.set_selected_idx(next_idx);
                                break;
                            }
                        }
                    }
                },
                AfterKeyInner::Cancel => {
                    self.field = Field::Ability;
                    self.saving_throw.set_active(false);
                },
                AfterKeyInner::Forward(key) => {
                    let KeyCode::Char(label) = key.code else {
                        return AfterKey::Stay;
                    };

                    let Some(idx) = LABELS.into_iter().position(|ch| ch == label) else {
                        return AfterKey::Stay;
                    };

                    if idx < self.combatants.len() {
                        self.set_selected_idx(idx);
                    }
                },
            }

            AfterKey::Stay
        } else {
            match self.input.handle_key(key) {
                AfterKeyInner::Handled => AfterKey::Stay,
                AfterKeyInner::Submit(value) => {
                    for data in &self.combatants {
                        let combatant_idx = data.idx;
                        let combatant = &mut tracker.combatants[combatant_idx];
                        combatant.damage(value, self.kind);
                    }
                    AfterKey::Exit
                },
                AfterKeyInner::Cancel => {
                    self.field = Field::RollSave;
                    self.saving_throw.set_active(true);
                    self.input.set_active(false);
                    AfterKey::Stay
                },
                AfterKeyInner::Forward(key) => {
                    let KeyCode::Char(label) = key.code else {
                        return AfterKey::Stay;
                    };

                    let label_to_option = LABELS
                        .into_iter()
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

    /// Change the selected combatant in the saving throw table.
    fn set_selected_idx(&mut self, new_idx: usize) {
        self.selected_idx = new_idx;

        let data = &self.combatants[new_idx];
        let save_value = data.save
            .map(|save| save.to_string())
            .unwrap_or(String::new());
        let save_mod = self.ability
            .map(|ability| save_modifier(ability, data.proficiencies, data.modifiers))
            .unwrap_or(0);
        self.saving_throw.set_value(save_value);
        self.saving_throw.set_prefix(fmt_dice_expr(save_mod));
    }
}
