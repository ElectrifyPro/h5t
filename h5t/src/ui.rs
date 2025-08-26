use bimap::BiMap;
use crate::{
    state::{AfterKey, ApplyCondition, ApplyDamage, State},
    theme::THEME,
    widgets::{max_combatants, CombatantBlock, StatBlock, Tracker as TrackerWidget},
};
use crossterm::event::{read, Event, KeyCode};
use h5t_core::{CombatantKind, Tracker};
use ratatui::{prelude::*, widgets::canvas::Canvas};
use std::{collections::HashSet, ops::{Deref, DerefMut}};

/// Labels used for label mode. The tracker will choose labels from this string in sequential
/// order.
///
/// The sequence of labels is simply the characters on a QUERTY keyboard, starting from the
/// top-left and moving down, then right. This keeps labels physically close to each other on the
/// keyboard.
pub(crate) const LABELS: &str = "qazwsxedcrfvtgbyhnujmik,ol.p;/[']";

/// The info block to show in the UI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InfoBlock {
    /// Show the combatant's full stat block, mostly useful for monsters.
    StatBlock,

    /// Show the combatant's current combat state.
    CombatantCard,
}

impl InfoBlock {
    /// Toggle the info block.
    pub fn toggle(&mut self) {
        *self = match self {
            InfoBlock::StatBlock => InfoBlock::CombatantCard,
            InfoBlock::CombatantCard => InfoBlock::StatBlock,
        };
    }
}

/// State passed to [`TrackerWidget`] to handle label mode.
#[derive(Clone, Debug, Default)]
pub struct LabelModeState {
    /// The labels to display next to each combatant.
    pub labels: BiMap<char, usize>,

    /// The labels that have been selected.
    pub selected: HashSet<char>,
}

/// A wrapper around a [`Tracker`] that handles UI-dependent logic, such as label mode.
pub struct Ui<B: Backend> {
    /// The terminal to draw to.
    pub terminal: Terminal<B>,

    /// The underlying tracker.
    pub tracker: Tracker,

    /// Which info block to show.
    info_block: InfoBlock,

    /// The currently active state.
    state: Option<State>,

    /// State for label mode.
    label_state: Option<LabelModeState>,
}

impl<B: Backend> Drop for Ui<B> {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

impl<B: Backend> Ui<B> {
    /// Wrap a [`Tracker`] in a new [`UiTracker`].
    pub fn new(terminal: Terminal<B>, tracker: Tracker) -> Self {
        Self {
            terminal,
            tracker,
            info_block: InfoBlock::CombatantCard,
            state: None,
            label_state: None,
        }
    }

    /// Run off the tracker until the user exits.
    pub fn run(&mut self) {
        loop {
            self.draw().unwrap();

            // wait for user input
            let Ok(Event::Key(key)) = read() else {
                continue;
            };

            // if a state is active, let it handle the input
            if let Some(mut state) = self.state.take() {
                match state.handle_key(key, &mut self.tracker) {
                    AfterKey::Exit => self.label_state = None,
                    AfterKey::Stay => self.state = Some(state),
                }
                continue;
            }

            match key.code {
                KeyCode::Char('c') => {
                    let selected = self.enter_label_mode();
                    if selected.is_empty() {
                        self.label_state = None;
                        continue;
                    }
                    self.state = Some(State::ApplyCondition(ApplyCondition::new(selected)));
                },
                KeyCode::Char('d') => {
                    let selected = self.enter_label_mode();
                    if selected.is_empty() {
                        self.label_state = None;
                        continue;
                    }
                    self.state = Some(State::ApplyDamage(ApplyDamage::new(selected)));
                },
                KeyCode::Char('a') => {
                    self.use_action();
                },
                KeyCode::Char('b') => {
                    self.use_bonus_action();
                },
                KeyCode::Char('r') => {
                    self.use_reaction();
                },
                KeyCode::Char('s') => {
                    self.info_block.toggle();
                },
                KeyCode::Char('n') => {
                    self.next_turn();
                },
                KeyCode::Char('q') => break,
                _ => (),
            }
        }
    }

    /// Draw the tracker to the terminal.
    pub fn draw(&mut self) -> std::io::Result<ratatui::CompletedFrame> {
        self.terminal.draw(|frame| {
            // clear the area
            frame.render_widget(
                Canvas::default()
                    .background_color(THEME.background.into())
                    .paint(|_| ()),
                frame.area(),
            );

            let layout = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]).split(frame.area());
            let [tracker_area, info_area] = [layout[0], layout[1]];

            // show tracker
            let tracker_widget = if let Some(label) = &self.label_state {
                TrackerWidget::with_labels(&self.tracker, label.clone())
            } else {
                TrackerWidget::new(&self.tracker)
            };
            frame.render_widget(tracker_widget, tracker_area);

            let combatant = self.tracker.current_combatant();
            if self.info_block == InfoBlock::StatBlock {
                // show stat block in place of the combatant card
                let CombatantKind::Monster(monster) = &combatant.kind;
                frame.render_widget(StatBlock::new(monster), info_area);
            } else {
                // show combatant card
                frame.render_widget(CombatantBlock::new(combatant), info_area);
            }

            let Some(state) = self.state.as_ref() else {
                return;
            };
            state.draw(frame);
        })
    }

    /// Enters label mode.
    ///
    /// Label mode is a special state where the user can quickly select one or more combatants
    /// to apply an action to. This works by displaying a label next to each combatant's name, and
    /// the user can press the corresponding key to toggle the label on or off.
    ///
    /// This function blocks until the user selects the combatants and presses the `Enter` key,
    /// returning mutable references to the selected combatants.
    pub fn enter_label_mode(&mut self) -> Vec<usize> {
        let size = self.terminal.size().unwrap();
        let num_combatants_in_view = max_combatants(size).min(self.combatants.len());

        // generate labels for all combatants in view
        let label_to_combatant_idx = (0..num_combatants_in_view)
            // .skip(self.turn) // TODO: change when pagination is implemented
            .map(|i| (LABELS.chars().nth(i).unwrap(), i))
            .collect::<BiMap<_, _>>();

        // watch for user-input and select combatants
        let mut selected_labels = HashSet::new();
        loop {
            // render tracker with labels
            self.label_state = Some(LabelModeState {
                labels: label_to_combatant_idx.clone(),
                selected: selected_labels.clone(),
            });
            self.draw().unwrap();

            // wait for user input
            if let Ok(Event::Key(key)) = read() {
                match key.code {
                    KeyCode::Esc => return vec![],
                    KeyCode::Enter => break,
                    KeyCode::Char(label) => {
                        if label_to_combatant_idx.contains_left(&label) {
                            if selected_labels.contains(&label) {
                                selected_labels.remove(&label);
                            } else {
                                selected_labels.insert(label);
                            }
                        }
                    },
                    _ => (),
                }
            }
        }

        // return selected combatants
        selected_labels
            .into_iter()
            .filter_map(|label| label_to_combatant_idx.get_by_left(&label).copied())
            .collect()
    }
}

impl<B: Backend> Deref for Ui<B> {
    type Target = Tracker;

    fn deref(&self) -> &Self::Target {
        &self.tracker
    }
}

impl<B: Backend> DerefMut for Ui<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tracker
    }
}

impl<B: Backend> Widget for Ui<B> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        TrackerWidget::new(&self.tracker).render(area, buf);
    }
}
