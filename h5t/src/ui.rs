use bimap::BiMap;
use crate::{selectable::Selectable, widgets::{max_combatants, popup::{Input, Multiselect}, CombatantBlock, StatBlock, Tracker as TrackerWidget}};
use crossterm::event::{read, Event, KeyCode};
use h5t_core::{CombatantKind, Condition, Tracker};
use ratatui::prelude::*;
use std::{collections::{HashMap, HashSet}, ops::{Deref, DerefMut}};

/// Labels used for label mode. The tracker will choose labels from this string in sequential
/// order.
///
/// The sequence of labels is simply the characters on a QUERTY keyboard, starting from the
/// top-left and moving down, then right. This keeps labels physically close to each other on the
/// keyboard.
pub(crate) const LABELS: &'static str = "qazwsxedcrfvtgbyhnujmik,ol.p;/[']";

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

/// State for applying conditions to combatants.
#[derive(Clone, Debug, Default)]
pub struct ConditionState {
    /// Whether the condition state is active.
    active: bool,

    /// The conditions to apply to combatants.
    conditions: HashSet<Condition>,

    /// Duration of the conditions.
    duration: ConditionDuration,
}

/// Duration of a condition.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ConditionDuration {
    /// The condition lasts until the end of the combatant's next turn.
    #[default]
    UntilNextTurn,

    /// The condition lasts for the given number of rounds.
    ///
    /// When the combatant's turn ends, the duration is decremented by one. When the duration
    /// is reduced to zero, the condition ends.
    Rounds(u32),
}

/// State for the input field.
#[derive(Clone, Debug, Default)]
pub struct InputState {
    /// Whether the input field is active.
    active: bool,

    /// Color of the input field.
    color: Color,

    /// The prompt to display above the input field.
    prompt: String,

    /// The value of the input field.
    value: String,
}

impl InputState {
    /// Enable a fresh input field.
    pub fn enable(&mut self, prompt: &str) {
        self.active = true;
        self.color = Color::Reset;
        self.prompt = prompt.to_string();
        self.value.clear();
    }

    /// Disable the input field.
    pub fn disable(&mut self) {
        self.active = false;
        self.color = Color::DarkGray;
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

    /// State for applying conditions to combatants.
    condition_state: ConditionState,

    /// State for the input field.
    input_state: InputState,

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
            condition_state: ConditionState::default(),
            input_state: InputState::default(),
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

            match key.code {
                KeyCode::Char('c') => {
                    // apply condition
                    // let selected = tracker.enter_label_mode();
                    let conditions = self.multi_select_enum("Select condition(s)");
                    panic!("conditions: {:?}", conditions);
                },
                KeyCode::Char('d') => {
                    // TEST: choose and damage a combatant
                    let selected = self.enter_label_mode();
                    let value = self.get_value::<i32>("Damage amount").unwrap();
                    for combatant_idx in selected {
                        let combatant = &mut self.tracker.combatants[combatant_idx];
                        combatant.damage(value);
                    }
                    self.label_state = None;
                    continue;
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

            if self.condition_state.active {
                frame.render_widget(Multiselect::new(
                    "Select condition(s)",
                    &self.condition_state.conditions,
                ), frame.area());
            }

            if self.input_state.active {
                frame.render_widget(Input::new(
                    self.input_state.color,
                    &self.input_state.prompt,
                    &self.input_state.value,
                    4,
                ), frame.area());
            }
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

    /// Get a value from the user.
    ///
    /// This function creates a visual prompt for the user to enter a value. The user can type in
    /// the value and press `Enter` to submit it.
    pub fn get_value<T: std::str::FromStr>(&mut self, prompt: &str) -> Result<T, T::Err> {
        self.input_state.enable(prompt);

        loop {
            self.draw().unwrap();

            // wait for user input
            if let Ok(Event::Key(key)) = read() {
                match key.code {
                    KeyCode::Enter => break,
                    KeyCode::Char(c) => self.input_state.value.push(c),
                    KeyCode::Backspace => { self.input_state.value.pop(); },
                    _ => (),
                }
            }

            let valid = self.input_state.value.trim().parse::<T>().is_ok();
            self.input_state.color = if valid { Color::Reset } else { Color::Red };
        }

        self.input_state.disable();
        self.input_state.value.trim().parse()
    }

    /// Create a multi-select prompt for an enum.
    ///
    /// This creates a popup with a list of options that the user can select from. The user can
    /// select multiple options (similar to label mode) and press `Enter` to submit their choices.
    pub fn multi_select_enum(&mut self, prompt: &str) -> Vec<Condition> {
        self.condition_state.active = true;
        self.condition_state.conditions.clear();

        // generate labels for all options in view
        let label_to_option = LABELS
            .chars()
            .zip(Condition::variants())
            .collect::<HashMap<_, _>>();

        // watch for user-input and select options
        loop {
            self.draw().unwrap();

            // wait for user input
            if let Ok(Event::Key(key)) = read() {
                match key.code {
                    KeyCode::Enter => break,
                    KeyCode::Char(label) => {
                        let selected = &mut self.condition_state.conditions;
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
            }
        }

        self.condition_state.active = false;

        // return selected options
        self.condition_state.conditions.iter().copied().collect()
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
