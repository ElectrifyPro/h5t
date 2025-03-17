pub mod apply_condition;
pub mod apply_damage;

pub use apply_condition::ApplyCondition;
pub use apply_damage::ApplyDamage;
use h5t_core::Tracker;

/// What to do after handling a key event.
#[derive(Default)]
pub enum AfterKey {
    /// Stay in the current state.
    #[default]
    Stay,

    /// Exit and hand control back to the main loop.
    Exit,
}

/// The current state the tracker is in. This encompasses states where an action is about to be
/// taken.
#[derive(Debug, Clone)]
pub enum State {
    /// Applying a condition to one or more combatants.
    ApplyCondition(ApplyCondition),

    /// Applying damage to one or more combatants.
    ApplyDamage(ApplyDamage),
}

impl State {
    /// Allow the state to draw itself.
    pub fn draw(&self, frame: &mut ratatui::Frame) {
        match self {
            Self::ApplyCondition(state) => state.draw(frame),
            Self::ApplyDamage(state) => state.draw(frame),
        }
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> AfterKey {
        match self {
            Self::ApplyCondition(state) => state.handle_key(key),
            Self::ApplyDamage(state) => state.handle_key(key),
        }
    }

    /// Apply the action to the tracker. This function is called when the state is exited.
    pub fn apply(self, tracker: &mut Tracker) {
        match self {
            Self::ApplyCondition(state) => state.apply(tracker),
            Self::ApplyDamage(state) => state.apply(tracker),
        }
    }
}
