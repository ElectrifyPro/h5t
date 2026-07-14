mod state;

use crate::{theme::{Rgb, THEME}, widgets::Setup as SetupWidget};
use crossterm::event::{read, Event, KeyCode};
use h5t_core::Combatant;
use ratatui::{prelude::*, widgets::canvas::Canvas};
use state::{AddCombatant, AfterKey, AssignGroup, NewGroup, RollInitiative, State};
use std::collections::HashMap;

/// The setup data for the battle.
#[derive(Debug)]
pub struct SetupInner {
    /// Groups that combatants can be assigned to.
    pub groups: HashMap<String, Rgb>,

    /// List of combatants to add to the battle in no particular order.
    pub combatants: Vec<Combatant>,
}

/// The setup view, used to setup and add combatants, and roll initiative order.
pub struct Setup<B: Backend> {
    /// The terminal to draw to.
    pub terminal: Terminal<B>,

    /// The setup data for the battle.
    inner: SetupInner,

    /// The currently active state.
    state: Option<State>,

    /// Index of the first combatant listed in the tracker, used to scroll through the initiative
    /// tracker.
    scroll_index: usize,
}

impl<B: Backend> Drop for Setup<B> {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

impl<B: Backend> Setup<B> {
    /// Create a new [`Setup`] view.
    pub fn new(terminal: Terminal<B>) -> Self {
        Self {
            terminal,
            inner: SetupInner {
                groups: HashMap::new(),
                combatants: vec![],
            },
            state: None,
            scroll_index: 0,
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
                match state.handle_key(key, &mut self.inner) {
                    AfterKey::Exit => (),
                    AfterKey::Stay => self.state = Some(state),
                }
                continue;
            }

            match key.code {
                KeyCode::Up => self.scroll_index = self.scroll_index.saturating_sub(1),
                KeyCode::Down => if self.scroll_index < self.inner.combatants.len() {
                    self.scroll_index += 1;
                },
                KeyCode::Char('a') => {
                    self.state = Some(State::AddCombatant(AddCombatant::new()));
                },
                KeyCode::Char('g') => {
                    self.state = Some(State::NewGroup(NewGroup::new()));
                },
                KeyCode::Char('G') => {
                    self.state = Some(State::AssignGroup(AssignGroup::new(&self.inner)));
                },
                KeyCode::Char('r') => {
                    self.state = Some(State::RollInitiative(RollInitiative::new(&self.inner)));
                },
                // roll initiative for all combatants
                KeyCode::Char('R') => for combatant in self.inner.combatants.iter_mut() {
                    let roll = rand::random::<u32>() % 20 + 1;
                    let dex_mod = combatant.scores().modifiers().dexterity;
                    combatant.initiative = Some(roll as i32 + dex_mod);
                },
                KeyCode::Char('s') => {
                    self.inner.combatants.sort_by(|a, b| {
                        match (a.initiative, b.initiative) {
                            // sort combatants with no initiative *before* those with a set
                            // initiative to make it obvious to user
                            (a @ Some(_), b @ None) | (a @ None, b @ Some(_)) => a.cmp(&b),

                            // higher initiative comes first
                            (a, b) => b.cmp(&a),
                        }
                    });
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

            frame.render_widget(
                SetupWidget::new(&self.inner, self.scroll_index),
                frame.area(),
            );

            let Some(state) = self.state.as_ref() else {
                return;
            };
            state.draw(frame);
        })
    }
}
