use bimap::BiMap;
use crate::{monster::MonsterCard, tracker::{max_combatants, TrackerWidget}};
use crossterm::event::{read, Event, KeyCode};
use h5t_core::{Combatant, CombatantKind, Tracker};
use ratatui::prelude::*;
use std::{collections::HashSet, ops::{Deref, DerefMut}};

/// Labels used for label mode. The tracker will choose labels from this string in sequential
/// order.
///
/// The sequence of labels is simply the characters on a QUERTY keyboard, starting from the
/// top-left and moving down, then right. This keeps labels physically close to each other on the
/// keyboard.
const LABELS: &'static str = "qazwsxedcrfvtgbyhnujmik,ol.p;/[']";

/// State passed to [`TrackerWidget`] to handle label mode.
#[derive(Debug, Default)]
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
}

impl<B: Backend> Drop for UiTracker<B> {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

impl<B: Backend> UiTracker<B> {
    /// Wrap a [`Tracker`] in a new [`UiTracker`].
    pub fn new(terminal: Terminal<B>, tracker: Tracker) -> Self {
        Self { terminal, tracker }
    }

    /// Draw the tracker to the terminal.
    pub fn draw(&mut self) -> std::io::Result<ratatui::CompletedFrame> {
        self.terminal.draw(|frame| {
            let layout = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]).split(frame.area());

            // print tracker
            let tracker = TrackerWidget::new(&self.tracker);
            frame.render_widget(tracker, layout[0]);

            // print a nice card
            let combatant = self.tracker.current_combatant();
            let CombatantKind::Monster(monster) = &combatant.kind;
            frame.render_widget(MonsterCard::new(monster), layout[1]);
        })
    }

    /// Draw the tracker to the terminal in label mode.
    pub fn draw_labels(&mut self, label: LabelModeState) -> std::io::Result<ratatui::CompletedFrame> {
        self.terminal.draw(|frame| {
            let layout = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]).split(frame.area());

            // print tracker
            let tracker = TrackerWidget::with_labels(&self.tracker, label);
            frame.render_widget(tracker, layout[0]);

            // print a nice card
            let combatant = self.tracker.current_combatant();
            let CombatantKind::Monster(monster) = &combatant.kind;
            frame.render_widget(MonsterCard::new(monster), layout[1]);
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
    pub fn enter_label_mode(&mut self) -> Vec<&mut Combatant> {
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
            self.draw_labels(LabelModeState {
                labels: label_to_combatant_idx.clone(),
                selected: selected_labels.clone(),
            }).unwrap();

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
        self.combatants // TODO: has to search through all combatants, not just the ones in view
            .iter_mut()
            .enumerate()
            .filter_map(|(i, combatant)| {
                let Some(label) = LABELS.chars().nth(i) else {
                    return None;
                };
                if selected_labels.contains(&label) {
                    Some(combatant)
                } else {
                    None
                }
            })
            .collect()
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
