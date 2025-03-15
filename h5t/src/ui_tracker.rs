use bimap::BiMap;
use crate::{monster::MonsterCard, tracker::{max_combatants, TrackerWidget}};
use crossterm::event::{read, Event, KeyCode};
use h5t_core::{CombatantKind, Tracker};
use ratatui::{prelude::*, widgets::*};
use std::{collections::HashSet, ops::{Deref, DerefMut}};

/// Labels used for label mode. The tracker will choose labels from this string in sequential
/// order.
///
/// The sequence of labels is simply the characters on a QUERTY keyboard, starting from the
/// top-left and moving down, then right. This keeps labels physically close to each other on the
/// keyboard.
const LABELS: &'static str = "qazwsxedcrfvtgbyhnujmik,ol.p;/[']";

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
pub struct UiTracker<B: Backend> {
    /// The terminal to draw to.
    pub terminal: Terminal<B>,

    /// The underlying tracker.
    pub tracker: Tracker,

    /// Whether to show the stat block for the current combatant.
    show_stat_block: bool,

    /// State for the input field.
    input_state: InputState,

    /// State for label mode.
    pub label_state: Option<LabelModeState>,
}

impl<B: Backend> Drop for UiTracker<B> {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

impl<B: Backend> UiTracker<B> {
    /// Wrap a [`Tracker`] in a new [`UiTracker`].
    pub fn new(terminal: Terminal<B>, tracker: Tracker) -> Self {
        Self {
            terminal,
            tracker,
            show_stat_block: false,
            input_state: InputState::default(),
            label_state: None,
        }
    }

    /// Toggles the view of the current combatant's stat block.
    pub fn toggle_stat_block(&mut self) {
        self.show_stat_block = !self.show_stat_block;
    }

    /// Draw the tracker to the terminal.
    pub fn draw(&mut self) -> std::io::Result<ratatui::CompletedFrame> {
        self.terminal.draw(|frame| {
            let layout = Layout::vertical([
                Constraint::Fill(1),
                // reserve the bottom 3 rows for the prompt and input
                Constraint::Length(3),
            ]).split(frame.area());
            let [main_area, input_area] = [layout[0], layout[1]];

            let layout = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]).split(main_area);
            let [tracker_area, card_area] = [layout[0], layout[1]];

            let tracker_widget = if let Some(label) = &self.label_state {
                TrackerWidget::with_labels(&self.tracker, label.clone())
            } else {
                TrackerWidget::new(&self.tracker)
            };

            if self.show_stat_block {
                // show tracker
                frame.render_widget(tracker_widget, tracker_area);

                // show stat block
                let combatant = self.tracker.current_combatant();
                let CombatantKind::Monster(monster) = &combatant.kind;
                frame.render_widget(MonsterCard::new(monster), card_area);
            } else {
                // show only the tracker in the combined main_area
                frame.render_widget(tracker_widget, main_area);
            }

            // draw bordered box for the input field
            let input_color = if self.input_state.active {
                self.input_state.color
            } else {
                Color::DarkGray // inactive color
            };
            frame.render_widget(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(input_color))
                    .title(&*self.input_state.prompt),
                input_area,
            );

            if self.input_state.active {
                let layout = Layout::default()
                    .constraints([Constraint::Length(1)])
                    .horizontal_margin(2)
                    .vertical_margin(1)
                    .split(input_area);

                // print input
                let input = Paragraph::new(&*self.input_state.value);
                frame.render_widget(input, layout[0]);
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
}

impl<B: Backend> Deref for UiTracker<B> {
    type Target = Tracker;

    fn deref(&self) -> &Self::Target {
        &self.tracker
    }
}

impl<B: Backend> DerefMut for UiTracker<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tracker
    }
}

impl<B: Backend> Widget for UiTracker<B> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        TrackerWidget::new(&self.tracker).render(area, buf);
    }
}
