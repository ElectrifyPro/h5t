use crate::{
    input::{AfterKey as AfterKeyInner, Charset, GetInput},
    theme::THEME,
    widgets::popup::Popup,
    Tracker,
};
use crossterm::event::{KeyEvent};
use h5t_core::{DeathSaveCount, Health};
use ratatui::{layout::Flex, prelude::*};
use super::AfterKey;

/// State for rolling a death saving throw for a downed combatant. It is automatically triggered for
/// downed players.
///
// TODO: currently applies to all combatants
#[derive(Clone, Debug)]
pub struct ApplyDeathSavingThrow {
    /// Helper to get the death saving throw roll.
    saving_throw: GetInput<i32>,

    /// The index of the combatant making death saving throws.
    ///
    /// This index should just point to [`Tracker::current_combatant_mut`].
    combatant: usize,

    /// The number of death save successes and failures the combatant has.
    counts: DeathSaveCount,
}

impl ApplyDeathSavingThrow {
    /// Create an [`ApplyDeathSavingThrow`] state with the initial state.
    pub fn new(combatant: usize, counts: DeathSaveCount) -> Self {
        Self {
            saving_throw: GetInput::new("Roll death save", 4, Charset::Numeric)
                .prefix("d20 ="),
            combatant,
            counts,
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let [save, outcome] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
        ])
            .flex(Flex::Center)
            .areas(area);

        self.saving_throw.draw(frame, save);

        let save_line = Line::from_iter([
            Span::raw("save result: "),
            match self.saving_throw.get_parsed() {
                Ok(1) => Span::raw("CRIT FAIL").fg(THEME.error),
                Ok(20) => Span::raw("CRIT PASS").fg(THEME.success),
                Ok(..=9) => Span::raw("FAIL").fg(THEME.error),
                Ok(10..) => Span::raw("PASS").fg(THEME.success),
                Err(_) => Span::raw("????").fg(THEME.dim().foreground),
            }
        ])
            .italic();
        Popup::new(THEME.foreground, None, true, save_line)
            .render(outcome, frame.buffer_mut());
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        match self.saving_throw.handle_key(key) {
            AfterKeyInner::Handled => (),
            AfterKeyInner::Submit(save_result) => {
                let combatant = &mut tracker.combatants[self.combatant];
                match save_result {
                    1 => self.counts.failures += 2,
                    20 => {
                        combatant.health = Health::Hp(1.try_into().unwrap());
                        return AfterKey::Exit;
                    },
                    ..=9 => self.counts.failures += 1,
                    10.. => self.counts.successes += 1,
                }

                if self.counts.successes >= 3 {
                    combatant.health = Health::Stabilized;
                } else if self.counts.failures >= 3 {
                    combatant.health = Health::Dead;
                } else {
                    combatant.health = Health::Downed(self.counts);
                }

                return AfterKey::Exit;
            },
            AfterKeyInner::Cancel => return AfterKey::Exit,
            AfterKeyInner::Forward(_) => (),
        }

        AfterKey::Stay
    }
}
