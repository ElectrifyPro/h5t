pub mod apply_condition;
pub mod apply_damage;
pub mod select_action;
pub mod use_movement;

pub use apply_condition::ApplyCondition;
pub use apply_damage::ApplyDamage;
pub use select_action::SelectAction;
pub use use_movement::UseMovement;
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

    /// Choosing an action to spend an action point on.
    SelectAction(SelectAction),

    /// Moving.
    UseMovement(UseMovement),
}

impl State {
    /// Allow the state to draw itself.
    pub fn draw(&self, frame: &mut ratatui::Frame) {
        match self {
            Self::ApplyCondition(state) => state.draw(frame),
            Self::ApplyDamage(state) => state.draw(frame),
            Self::SelectAction(state) => state.draw(frame),
            Self::UseMovement(state) => state.draw(frame),
        }
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        tracker: &mut Tracker,
    ) -> AfterKey {
        match self {
            Self::ApplyCondition(state) => state.handle_key(key, tracker),
            Self::ApplyDamage(state) => state.handle_key(key, tracker),
            Self::SelectAction(state) => state.handle_key(key, tracker),
            Self::UseMovement(state) => state.handle_key(key, tracker),
        }
    }
}
