mod state;

use bimap::BiMap;
use crate::{
    theme::{Rgb, THEME},
    view::LABELS,
    widgets::{max_combatants, CombatantBlock, StatBlock, Tracker as TrackerWidget},
};
use crossterm::event::{read, Event, KeyCode};
use h5t_core::{Health, Tracker};
use ratatui::{prelude::*, widgets::canvas::Canvas};
use state::{
    AfterKey,
    ApplyCondition,
    ApplyDamage,
    ApplyDeathSavingThrow,
    ApplySavingThrowDamage,
    SelectAction,
    State,
    UseMovement,
};
use std::{collections::{HashMap, HashSet}, ops::{Deref, DerefMut}};

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
pub struct Battle {
    /// The underlying tracker.
    pub tracker: Tracker,

    /// Groups that combatants can be assigned to.
    pub groups: HashMap<String, Rgb>,

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

impl Battle {
    /// Wrap a [`Tracker`] in a new [`Battle`].
    pub fn new(tracker: Tracker, groups: HashMap<String, Rgb>) -> Self {
        Self {
            tracker,
            groups,
            info_block: InfoBlock::CombatantCard,
            state: None,
            scroll_index: 0,
            label_state: None,
        }
    }

    /// Run off the tracker until the user exits.
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) {
        loop {
            self.draw(terminal).unwrap();

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
                    let selected = self.enter_label_mode(terminal);
                    if selected.is_empty() {
                        continue;
                    }
                    self.state = Some(State::ApplyCondition(ApplyCondition::new(selected)));
                },
                KeyCode::Char('d') => {
                    let selected = self.enter_label_mode(terminal);
                    if selected.is_empty() {
                        continue;
                    }
                    self.state = Some(State::ApplyDamage(ApplyDamage::new(selected)));
                },
                KeyCode::Char('D') => {
                    let selected = self.enter_label_mode(terminal);
                    if selected.is_empty() {
                        continue;
                    }

                    let combatant_iter = selected
                        .into_iter()
                        .map(|idx| (idx, &self.combatants[idx]));
                    self.state = Some(State::ApplySavingThrowDamage(ApplySavingThrowDamage::new(
                        combatant_iter,
                        &self.groups,
                    )));
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

                    // trigger death save state
                    let current_health = self.current_combatant().health;
                    if let Health::Downed(counts) = current_health {
                        self.state = Some(State::ApplyDeathSavingThrow(ApplyDeathSavingThrow::new(
                            self.turn,
                            counts,
                        )));
                    }
                },
                KeyCode::Char('N') => {
                    // skip all dead combatants
                    loop {
                        self.next_turn();
                        // TODO: loop forever if all dead
                        if self.current_combatant().health.active() {
                            break;
                        }
                    }

                    // trigger death save state
                    let current_health = self.current_combatant().health;
                    if let Health::Downed(counts) = current_health {
                        self.state = Some(State::ApplyDeathSavingThrow(ApplyDeathSavingThrow::new(
                            self.turn,
                            counts,
                        )));
                    }
                },
                KeyCode::Char('q') => break,
                _ => (),
            }
        }
    }

    /// Draw the tracker to the terminal.
    pub fn draw<'a, B: Backend>(
        &mut self,
        terminal: &'a mut Terminal<B>,
    ) -> Result<ratatui::CompletedFrame<'a>, B::Error> {
        terminal.draw(|frame| {
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
                TrackerWidget::with_labels(&self.tracker, &self.groups, self.scroll_index, label)
            } else {
                TrackerWidget::new(&self.tracker, &self.groups, self.scroll_index)
            };
            frame.render_widget(tracker_widget, tracker_area);

            let combatant = self.tracker.current_combatant();
            if self.info_block == InfoBlock::StatBlock {
                // show stat block in place of the combatant card
                frame.render_widget(StatBlock::new(&combatant.kind), info_area);
            } else {
                // show combatant card
                let group_color = combatant.group
                        .as_ref()
                        .and_then(|group| self.groups.get(group).copied());
                frame.render_widget(CombatantBlock::new(group_color, combatant), info_area);
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
    pub fn enter_label_mode<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> HashSet<usize> {
        let size = terminal.size().unwrap();

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
            self.draw(terminal).unwrap();
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

// TODO: Deref really does not make sense
impl Deref for Battle {
    type Target = Tracker;

    fn deref(&self) -> &Self::Target {
        &self.tracker
    }
}

impl DerefMut for Battle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tracker
    }
}

impl Widget for Battle {
    fn render(self, area: Rect, buf: &mut Buffer) {
        TrackerWidget::new(&self.tracker, &self.groups, self.scroll_index).render(area, buf);
    }
}
