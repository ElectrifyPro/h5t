pub mod ability;
pub mod monster;

pub use ability::{Ability, score_to_modifier};
pub use monster::Monster;

/// A combatant in the initiative tracker.
///
/// Combatants can include player characters, monsters, NPCs, etc.
#[derive(Debug)]
pub enum Combatant {
    /// Pre-made monster.
    Monster(Monster),
}

impl From<Monster> for Combatant {
    fn from(monster: Monster) -> Self {
        Self::Monster(monster)
    }
}

/// The core initiative tracker.
///
/// It handles the order of play and tracks every important detail, such as the current turn,
/// conditions on each combatant, actions taken, etc.
#[derive(Debug)]
pub struct Tracker {
    /// The index of the combatant that is taking their turn.
    pub turn: usize,

    /// The current round, starting at 0 (to mean the first round).
    pub round: usize,

    /// The list of combatants.
    pub combatants: Vec<Combatant>,
}

impl Tracker {
    /// Create a new initiative tracker with the given combatants.
    pub fn new(combatants: impl Into<Vec<Combatant>>) -> Self {
        Self {
            turn: 0,
            round: 0,
            combatants: combatants.into(),
        }
    }

    /// Advance the tracker to the next combatant's turn.
    pub fn next_turn(&mut self) {
        self.turn = (self.turn + 1) % self.combatants.len();
        if self.turn == 0 {
            self.round += 1;
        }
    }

    /// Get the combatant that is currently taking their turn.
    pub fn current_combatant(&self) -> &Combatant {
        &self.combatants[self.turn]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure that the tracker advances turns correctly.
    #[test]
    fn test_tracker_next_turn() {
        let mut tracker = Tracker::new(vec![
            Combatant::Monster(Monster {
                index: "goblin".to_string(),
                name: "Goblin".to_string(),
                ..Default::default()
            }),
            Combatant::Monster(Monster {
                index: "ogre".to_string(),
                name: "Ogre".to_string(),
                ..Default::default()
            }),
            Combatant::Monster(Monster {
                index: "tarrasque".to_string(),
                name: "Tarrasque".to_string(),
                ..Default::default()
            }),
        ]);

        assert_eq!(tracker.turn, 0);
        assert_eq!(tracker.round, 0);

        tracker.next_turn();
        assert_eq!(tracker.turn, 1);
        assert_eq!(tracker.round, 0);

        tracker.next_turn();
        assert_eq!(tracker.turn, 2);
        assert_eq!(tracker.round, 0);

        tracker.next_turn();
        assert_eq!(tracker.turn, 0);
        assert_eq!(tracker.round, 1);
    }
}
