pub mod ability;
pub mod monster;

pub use ability::{Ability, score_to_modifier};
pub use monster::Monster;

/// A combatant in the initiative tracker.
///
/// Combatants can include player characters, monsters, NPCs, etc.
#[derive(Debug)]
pub struct Combatant {
    /// The kind of combatant.
    pub kind: CombatantKind,

    /// The combatant's current hit points.
    pub hit_points: i32,
}

impl From<CombatantKind> for Combatant {
    fn from(kind: CombatantKind) -> Self {
        match kind {
            CombatantKind::Monster(monster) => monster.into(),
        }
    }
}

impl Combatant {
    /// Returns the combatant's name.
    pub fn name(&self) -> &str {
        match &self.kind {
            CombatantKind::Monster(monster) => &monster.name,
        }
    }

    /// Returns the combatant's maximum hit points.
    pub fn max_hit_points(&self) -> i32 {
        match &self.kind {
            CombatantKind::Monster(monster) => monster.hit_points,
        }
    }
}

/// A kind of combatant.
#[derive(Debug)]
pub enum CombatantKind {
    /// Pre-made monster.
    Monster(Monster),
}

impl From<Monster> for CombatantKind {
    fn from(monster: Monster) -> Self {
        Self::Monster(monster)
    }
}

impl From<Monster> for Combatant {
    fn from(monster: Monster) -> Self {
        Self {
            hit_points: monster.hit_points,
            kind: monster.into(),
        }
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
            Monster {
                index: "goblin".to_string(),
                name: "Goblin".to_string(),
                ..Default::default()
            }.into(),
            Monster {
                index: "ogre".to_string(),
                name: "Ogre".to_string(),
                ..Default::default()
            }.into(),
            Monster {
                index: "tarrasque".to_string(),
                name: "Tarrasque".to_string(),
                ..Default::default()
            }.into(),
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
