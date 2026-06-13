pub mod add_combatant;

pub use add_combatant::AddCombatant;

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

            /// Handle a key event and apply any needed changes to the combatant list.
            pub fn handle_key(
                &mut self,
                key: crossterm::event::KeyEvent,
                combatants: &mut Vec<h5t_core::Combatant>,
            ) -> AfterKey {
                match self {
                    $(
                        Self::$state_name(state) => state.handle_key(key, combatants),
                    )+
                }
            }
        }
    }
}

create_state!(
    "Adding a combatant to the combat.", AddCombatant;
);
