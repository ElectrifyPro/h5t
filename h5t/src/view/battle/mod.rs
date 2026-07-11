mod state;

use bimap::BiMap;
use crate::{
    theme::THEME,
    view::LABELS,
    widgets::{max_combatants, CombatantBlock, StatBlock, Tracker as TrackerWidget},
};
use crossterm::event::{read, Event, KeyCode};
use h5t_core::Tracker;
use ratatui::{prelude::*, widgets::canvas::Canvas};
use state::{AfterKey, ApplyCondition, ApplyDamage, ApplySavingThrowDamage, SelectAction, State, UseMovement};
use std::{collections::HashSet, ops::{Deref, DerefMut}};

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

    /// Indices of all combatants that have been selected.
    pub selected_combatants: HashSet<usize>,
}

/// The battle view, used in combat or when strict initiative order is needed.
///
/// The underlying [`Tracker`] is used for rules management.
pub struct Battle<B: Backend> {
    /// The terminal to draw to.
    pub terminal: Terminal<B>,

    /// The underlying tracker.
    pub tracker: Tracker,

    /// Which info block to show.
    info_block: InfoBlock,

    /// The currently active state.
    state: Option<State>,

    /// Index of the first combatant listed in the tracker, used to scroll through the initiative
    /// tracker.
    scroll_index: usize,

    /// The current label mode state, held over while a state is being processed. This allows the
    /// tracker to keep highlighting selected combatants during the state's execution.
    label_state: Option<LabelModeState>,
}

impl<B: Backend> Drop for Battle<B> {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

impl<B: Backend> Battle<B> {
    /// Wrap a [`Tracker`] in a new [`Battle`].
    pub fn new(terminal: Terminal<B>, tracker: Tracker) -> Self {
        Self {
            terminal,
            tracker,
            info_block: InfoBlock::CombatantCard,
            state: None,
            scroll_index: 0,
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
                KeyCode::Up => self.scroll_index = self.scroll_index.saturating_sub(1),
                KeyCode::Down => if self.scroll_index < self.tracker.combatants.len() {
                    self.scroll_index += 1;
                },
                KeyCode::Char('c') => {
                    let selected = self.enter_label_mode();
                    if selected.is_empty() {
                        continue;
                    }
                    self.state = Some(State::ApplyCondition(ApplyCondition::new(selected)));
                },
                KeyCode::Char('d') => {
                    let selected = self.enter_label_mode();
                    if selected.is_empty() {
                        continue;
                    }
                    self.state = Some(State::ApplyDamage(ApplyDamage::new(selected)));
                },
                KeyCode::Char('D') => {
                    let selected = self.enter_label_mode();
                    if selected.is_empty() {
                        continue;
                    }

                    let data_iter = selected
                        .into_iter()
                        .map(|idx| (idx, &self.combatants[idx]));
                    self.state = Some(State::ApplySavingThrowDamage(ApplySavingThrowDamage::new(data_iter)));
                },
                KeyCode::Char('m') => {
                    self.state = Some(State::UseMovement(UseMovement::new()));
                },
                KeyCode::Char('a') => {
                    // TODO: should open UI to list all things(?) that consume resource: Action
                    let combatant = self.current_combatant();
                    self.state = Some(State::SelectAction(SelectAction::new(
                        combatant.actions(),
                        combatant.resource_pool.clone(),
                    )));
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
                KeyCode::Char('N') => {
                    // skip all dead combatants
                    loop {
                        self.next_turn();
                        if self.current_combatant().hit_points > 0 {
                            break;
                        }
                    }
                },
                KeyCode::Char('q') => break,
                _ => (),
            }
        }
    }

    /// Draw the tracker to the terminal.
    pub fn draw(&mut self) -> Result<ratatui::CompletedFrame<'_>, B::Error> {
        self.terminal.draw(|frame| {
            // clear the area
            frame.render_widget(
                Canvas::default()
                    .background_color(THEME.background.into())
                    .paint(|_| ()),
                frame.area(),
            );

            let [tracker_area, info_area] = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]).areas(frame.area());

            // show tracker
            let tracker_widget = if let Some(label) = self.label_state.as_ref() {
                TrackerWidget::with_labels(&self.tracker, self.scroll_index, label)
            } else {
                TrackerWidget::new(&self.tracker, self.scroll_index)
            };
            frame.render_widget(tracker_widget, tracker_area);

            let combatant = self.tracker.current_combatant();
            if self.info_block == InfoBlock::StatBlock {
                // show stat block in place of the combatant card
                frame.render_widget(StatBlock::new(&combatant.kind), info_area);
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
    /// Label mode is a special state where the user can quickly select one or more combatants to
    /// apply an action to. This works by displaying a label next to each combatant's name, and the
    /// user can press the corresponding key to toggle the label on or off.
    ///
    /// This function blocks until the user selects the combatants and presses the `Enter` key, and
    /// returns the indices of the selected combatants.
    pub fn enter_label_mode(&mut self) -> HashSet<usize> {
        let size = self.terminal.size().unwrap();

        // map as many labels as possible to combatants in the visible window. the window will never
        // be larger than the maximum possible number of combatants.
        let make_label_window = |start_index: usize| {
            LABELS
                .into_iter()
                .zip(start_index..)
                .take(max_combatants(size))
                .collect::<BiMap<_, _>>()
        };

        // for all combatants in view, generate as many labels as we can for them
        let mut current_start_index = self.scroll_index;
        let mut label_state = LabelModeState {
            labels: make_label_window(current_start_index),
            selected_combatants: HashSet::new(),
        };

        // watch for user-input and select combatants
        loop {
            // render tracker with labels
            self.label_state = Some(label_state);
            self.draw().unwrap();
            label_state = self.label_state.take().unwrap();

            // wait for user input
            if let Ok(Event::Key(key)) = read() {
                match key.code {
                    KeyCode::Esc => return HashSet::new(),
                    KeyCode::Enter => break,
                    KeyCode::BackTab => {
                        // shift + tab = move label window backwards
                        // shift window back by its size so there are no overlaps
                        // TODO: windows can overlap near the beginning of the initiative list.
                        current_start_index = current_start_index
                            .saturating_sub(label_state.labels.len());
                        label_state.labels = make_label_window(current_start_index);

                        // does the window begin outside the visible range of combatants?
                        // TODO: won't crash but this makes me feel ugly
                        if self.scroll_index > 0 {
                            let window_start_index = current_start_index;
                            let last_combatant_prev_page_idx = self.scroll_index - 1;
                            if window_start_index <= last_combatant_prev_page_idx {
                                // if so, scroll so we can see it
                                self.scroll_index = (window_start_index + label_state.labels.len())
                                    .saturating_sub(max_combatants(size));
                            }
                        }
                    },
                    KeyCode::Tab => {
                        // tab = move label window forewards
                        // advance window by its size so there are no overlaps
                        // TODO: windows can overlap near the end of the initiative list. also, this
                        // will likely crash. not tested yet
                        current_start_index = (current_start_index + label_state.labels.len())
                            .min(self.combatants.len() - label_state.labels.len());
                        label_state.labels = make_label_window(current_start_index);

                        // does the window end outside the visible range of combatants?
                        let window_end_index = current_start_index + label_state.labels.len() - 1;
                        let first_combatant_next_page_idx = self.scroll_index + max_combatants(size);
                        if window_end_index >= first_combatant_next_page_idx {
                            // if so, scroll so we can see it
                            self.scroll_index = current_start_index;
                        }
                    },
                    KeyCode::Char(label) => {
                        if let Some(combatant) = label_state.labels.get_by_left(&label) {
                            if label_state.selected_combatants.contains(combatant) {
                                label_state.selected_combatants.remove(combatant);
                            } else {
                                label_state.selected_combatants.insert(*combatant);
                            }
                        }
                    },
                    _ => (),
                }
            }
        }

        let selected_combatants = label_state.selected_combatants.clone();

        if !label_state.selected_combatants.is_empty() {
            self.label_state = Some(label_state);
        }

        selected_combatants
    }
}

impl<B: Backend> Deref for Battle<B> {
    type Target = Tracker;

    fn deref(&self) -> &Self::Target {
        &self.tracker
    }
}

impl<B: Backend> DerefMut for Battle<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tracker
    }
}

impl<B: Backend> Widget for Battle<B> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        TrackerWidget::new(&self.tracker, self.scroll_index).render(area, buf);
    }
}
