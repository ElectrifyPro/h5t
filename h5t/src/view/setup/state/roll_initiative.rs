use crate::{
    input::{AfterKey as AfterKeyInner, Charset, GetInput},
    theme::THEME,
    view::{LABELS, setup::SetupInner},
    widgets::{popup::Popup, SizedTable},
};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::{ability::Modifier, Combatant};
use ratatui::{layout::Flex, prelude::*, widgets::*};

use super::AfterKey;

/// Formats a dice expression in the form `d20 +/- <modifier>`.
///
/// TODO: copied from apply_saving_throw_damage
fn fmt_dice_expr(modifier: i32) -> String {
    let operator = match modifier.signum() {
        ..=-1 => '-',
        0 => return "d20 =".to_string(),
        1.. => '+',
    };
    let absolute_value = modifier.abs();
    format!("d20 {} {} =", operator, absolute_value)
}

/// Combatant data needed.
#[derive(Clone, Debug, Default)]
struct CombatantData {
    /// The index of the combatant in the setup view.
    idx: usize,

    /// The combatant's name.
    name: String,

    /// The combatant's dexterity modifier.
    dexterity: Modifier,

    /// The combatant's initiative roll.
    initiative: Option<i32>,
}

impl CombatantData {
    /// Create a [`CombatantData`] with a combatant.
    fn new((idx, combatant): (usize, &Combatant)) -> Self {
        Self {
            idx,
            name: combatant.name().to_string(),
            dexterity: combatant.scores().modifiers().dexterity,
            initiative: None,
        }
    }
}

/// State for rolling initiative for each combatant.
#[derive(Clone, Debug)]
pub struct RollInitiative {
    /// The combatants to roll initiative for.
    combatants: Vec<CombatantData>,

    /// Helper to get the initiative value from the user.
    initiative: GetInput<i32>,

    /// The combatant within the list of targeted combatants (`self.combatants`) we are rolling
    /// initiative for.
    selected_idx: usize,
}

impl RollInitiative {
    /// Create a [`RollInitiative`] state with the given setup state.
    pub fn new(inner: &SetupInner) -> Self {
        Self {
            combatants: inner.combatants
                .iter()
                .enumerate()
                .map(CombatantData::new)
                .collect(),
            initiative: GetInput::new("Initiative", 3, Charset::Numeric)
                .prefix("d20 ="),
            selected_idx: 0,
        }

    }

    /// Change the selected combatant.
    fn set_selected_idx(&mut self, new_idx: usize) {
        self.selected_idx = new_idx;

        let data = &self.combatants[new_idx];
        let initiative_value = data.initiative
            .map(|save| save.to_string())
            .unwrap_or(String::new());
        self.initiative.set_value(initiative_value);
        self.initiative.set_prefix(fmt_dice_expr(data.dexterity));
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let [initiative_input, combatant_table] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3 + self.combatants.len() as u16),
        ])
            .flex(Flex::Center)
            .areas(frame.area());

        self.initiative.draw(frame, initiative_input);

        let longest_combatant_name = self.combatants.iter()
            .map(|data| data.name.len())
            .max();
        let table = SizedTable::new(
            LABELS.into_iter()
                .zip(&self.combatants)
                .enumerate()
                .map(|(idx, (label, data))| {
                    let mut style = Style::default()
                        .fg(THEME.foreground.into());

                    if self.selected_idx == idx {
                        style = style.bg(THEME.select.into());
                    }

                    Row::new([
                        Text::from(format!("{}", label)).bold(),
                        Text::raw(&data.name),
                        Text::raw(fmt_dice_expr(data.dexterity)),
                        Text::raw(data.initiative.map(|value| value.to_string()).unwrap_or(String::new())),
                    ])
                        .style(style)
                }),
            [
                2,
                2 + longest_combatant_name.unwrap_or(0).max("Combatant".len()) as u16,
                2 + "Expression".len() as u16,
                2 + "Initiative".len() as u16,
            ],
            0,
        )
            .header(
                Row::new([
                    " ",
                    "Combatant",
                    "Expression",
                    "Initiative",
                ])
                    .style(THEME.foreground)
                    .bold(),
            );
        let popup = Popup::new(THEME.foreground, Some("Roll initiative"), true, table);
        popup.render(combatant_table, frame.buffer_mut());
    }

    /// Handle a key event and apply any needed changes to the combatant list.
    pub fn handle_key(&mut self, key: KeyEvent, inner: &mut SetupInner) -> AfterKey {
        match self.initiative.handle_key(key) {
            AfterKeyInner::Handled => {
                let Ok(save) = self.initiative.get_parsed() else {
                    return AfterKey::Stay;
                };

                let combatant = &mut self.combatants[self.selected_idx];
                combatant.initiative = Some(save);
            },
            AfterKeyInner::Submit(initiative) => {
                let combatant = &mut self.combatants[self.selected_idx];
                combatant.initiative = Some(initiative);

                // if all combatants have an initiative value, we can stop here
                if self.combatants.iter().all(|data| data.initiative.is_some()) {
                    for data in self.combatants.iter() {
                        inner.combatants[data.idx].initiative = data.initiative;
                    }
                    return AfterKey::Exit;
                } else {
                    // move to next combatant with no initiative set
                    let mut next_idx = self.selected_idx;
                    loop {
                        next_idx = (next_idx + 1) % self.combatants.len();
                        if self.combatants[next_idx].initiative.is_none() {
                            self.set_selected_idx(next_idx);
                            break;
                        }
                    }
                }
            },
            AfterKeyInner::Cancel => return AfterKey::Exit,
            AfterKeyInner::Forward(event) => {
                match event.code {
                    // roll initiative for all combatants (also possible outside of this state)
                    KeyCode::Char('R') => for (idx, combatant) in self.combatants.iter_mut().enumerate() {
                        let roll = rand::random::<u32>() % 20 + 1;
                        let dex_mod = combatant.dexterity;
                        let new_initiative = roll as i32 + dex_mod;
                        combatant.initiative = Some(new_initiative);

                        // also update the input field for the currently selected combatant
                        if idx == self.selected_idx {
                            self.initiative.set_value(new_initiative.to_string());
                        }
                    },
                    KeyCode::Char(key) => {
                        let Some(idx) = LABELS.into_iter().position(|ch| ch == key) else {
                            return AfterKey::Stay;
                        };

                        if idx < self.combatants.len() {
                            self.set_selected_idx(idx);
                        }
                    },
                    _ => (),
                }
            },
        }

        AfterKey::Stay
    }
}
