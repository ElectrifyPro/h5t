pub mod apply_condition;
pub mod apply_damage;
pub mod select_action;
pub mod use_movement;

pub use apply_condition::ApplyCondition;
pub use apply_damage::ApplyDamage;
pub use select_action::SelectAction;
pub use use_movement::UseMovement;

/// What to do after handling a key event.
#[derive(Default)]
pub enum AfterKey {
    /// Stay in the current state.
    #[default]
    Stay,

    /// Exit and hand control back to the main loop.
    Exit,
}

/// Creates the [`State`] enum.
macro_rules! create_state {
    ($($doc:literal, $state_name:ident);+ $(;)?) => {
        /// The current state the tracker is in. This encompasses states where an action is
        /// about to be taken.
        #[derive(Debug, Clone)]
        pub enum State {
            $(
                #[doc = $doc]
                $state_name($state_name),
            )+
        }

        impl State {
            /// Allow the state to draw itself.
            pub fn draw(&self, frame: &mut ratatui::Frame) {
                match self {
                    $(
                        Self::$state_name(state) => state.draw(frame),
                    )+
                }
            }

            /// Handle a key event and apply any needed changes to the tracker.
            pub fn handle_key(
                &mut self,
                key: crossterm::event::KeyEvent,
                tracker: &mut h5t_core::Tracker,
            ) -> AfterKey {
                match self {
                    $(
                        Self::$state_name(state) => state.handle_key(key, tracker),
                    )+
                }
            }
        }
    }
}

create_state!(
    "Applying a condition to one or more combatants.", ApplyCondition;
    "Applying damage to one or more combatants.", ApplyDamage;
    "Choosing an action to spend an action point on.", SelectAction;
    "Moving.", UseMovement;
);
