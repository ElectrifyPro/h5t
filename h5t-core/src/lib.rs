pub mod ability;
pub mod monster;

use ability::Modifier;
pub use ability::{Ability, score_to_modifier};
pub use monster::Monster;
use monster::Speed;

/// The number of actions, bonus actions, and reactions a combatant has.
#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub actions: u32,
    pub bonus_actions: u32,
    pub reactions: u32,
}

/// By default, a combatant has one action, one bonus action, and one reaction.
impl Default for Action {
    fn default() -> Self {
        Self {
            actions: 1,
            bonus_actions: 1,
            reactions: 1,
        }
    }
}

/// A combatant in the initiative tracker.
///
/// Combatants can include player characters, monsters, NPCs, etc.
#[derive(Debug)]
pub struct Combatant {
    /// The kind of combatant.
    pub kind: CombatantKind,

    /// The combatant's current hit points.
    pub hit_points: i32,

    /// The actions available to the combatant.
    pub actions: Action,
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

    /// Returns the combatant's main armor class.
    pub fn armor_class(&self) -> u32 {
        match &self.kind {
            CombatantKind::Monster(monster) => monster.armor_class.value,
        }
    }

    /// Returns the combatant's speed.
    pub fn speed(&self) -> &Speed {
        match &self.kind {
            CombatantKind::Monster(monster) => &monster.speed,
        }
    }

    /// Returns the combatant's maximum hit points.
    pub fn max_hit_points(&self) -> i32 {
        match &self.kind {
            CombatantKind::Monster(monster) => monster.hit_points,
        }
    }

    /// Returns the combatant's proficiency bonus.
    pub fn proficiency_bonus(&self) -> Modifier {
        match &self.kind {
            CombatantKind::Monster(monster) => monster.proficiency_bonus,
        }
    }

    /// Damage the combatant by the given amount.
    ///
    /// The amount will not saturate to 0, meaning the combatant can have negative hit points.
    pub fn damage(&mut self, amount: i32) {
        self.hit_points -= amount;
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
            actions: Action::default(),
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

    /// Use an action for the current combatant. Returns `true` if the action was used, or `false`
    /// if the combatant had no actions left to use.
    ///
    /// This function only decrements the number of actions available to the combatant, meaning the
    /// combat log will not display any information about the action taken.
    pub fn use_action(&mut self) -> bool {
        let count = &mut self.combatants[self.turn].actions.actions;
        if *count == 0 {
            return false;
        }
        *count = count.saturating_sub(1);
        true
    }

    /// Use a bonus action for the current combatant. Returns `true` if the bonus action was used,
    /// or `false` if the combatant had no bonus actions left to use.
    ///
    /// This function only decrements the number of bonus actions available to the combatant,
    /// meaning the combat log will not display any information about the bonus action taken.
    pub fn use_bonus_action(&mut self) -> bool {
        let count = &mut self.combatants[self.turn].actions.bonus_actions;
        if *count == 0 {
            return false;
        }
        *count = count.saturating_sub(1);
        true
    }

    /// Use a reaction for the current combatant. Returns `true` if the reaction was used, or
    /// `false` if the combatant had no reactions left to use.
    ///
    /// This function only decrements the number of reactions available to the combatant, meaning
    /// the combat log will not display any information about the reaction taken.
    pub fn use_reaction(&mut self) -> bool {
        let count = &mut self.combatants[self.turn].actions.reactions;
        if *count == 0 {
            return false;
        }
        *count = count.saturating_sub(1);
        true
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
