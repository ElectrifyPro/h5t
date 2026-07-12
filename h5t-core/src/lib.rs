pub mod ability;
pub mod action;
pub mod character;
pub mod class;
pub mod condition;
pub mod damage;
pub mod monster;
pub mod resource;
pub mod speed;
pub mod spell;

use ability::{Modifier, Proficiencies, Score};
pub use ability::{Ability, score_to_modifier};
pub use action::Action;
pub use character::Character;
pub use class::ClassKind;
pub use condition::{Condition, ConditionKind, ConditionDuration};
pub use damage::{DamageKind, MagicKind};
use enumset::EnumSet;
pub use monster::Monster;
use resource::{BonusAction, Reaction, Resource, ResourcePool};
pub use speed::Speed;
pub use spell::Spell;
use std::{borrow::Cow, collections::HashMap};

/// Generic ID, identifying a resource or action.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(Cow<'static, str>);

/// A combatant in the initiative tracker.
///
/// Combatants can include player characters, monsters, NPCs, etc.
#[derive(Debug)]
pub struct Combatant {
    /// The initiative value of the combatant.
    ///
    /// This value is only used when sorting the initiative order. The user can override initiative
    /// order at any time.
    pub initiative: Option<i32>,

    /// Identifier for the group the combatant is in.
    ///
    /// This can be used to clearly distinguish between players and NPCs, teams involved in a
    /// combat, etc.
    pub group: String,

    /// The kind of creature the combatant is.
    pub kind: CombatantKind,

    /// The combatant's conditions.
    pub conditions: Vec<Condition>,

    /// The combatant's current hit points.
    pub hit_points: i32,

    /// The number of resources available to the combatant, including action count, bonus action
    /// count, reaction count, and resources granted by classes (e.g. Superiority dice) and spells
    /// (e.g. Haste action).
    pub resource_pool: ResourcePool,
}

impl From<CombatantKind> for Combatant {
    fn from(kind: CombatantKind) -> Self {
        match kind {
            CombatantKind::Character(character) => character.into(),
            CombatantKind::Monster(monster) => monster.into(),
        }
    }
}

/// Makes trivial `impl`s of common getter functions on [`Combatant`]s.
macro_rules! combatant_impls {
    ($($doc:literal, $fn_name:ident, $return_type:ty);+ $(;)?) => {
        $(
            #[doc = $doc]
            pub fn $fn_name(&self) -> $return_type {
                self.kind.$fn_name()
            }
        )+
    }
}

impl Combatant {
    combatant_impls!(
        "Returns the combatant's name.", name, &str;
        "Returns the combatant's ability scores.", scores, Ability<Score>;
        "Returns the combatant's proficiencies.", proficiencies, Proficiencies;
        "Returns the combatant's main armor class.", armor_class, u32;
        "Returns the combatant's main base speed.", speed, &Speed;
        "Returns the combatant's maximum hit points.", max_hit_points, i32;
        "Returns the combatant's damage vulnerabilities.",
        damage_vulnerabilities, &HashMap<DamageKind, EnumSet<MagicKind>>;
        "Returns the combatant's damage resistances.",
        damage_resistances, &HashMap<DamageKind, EnumSet<MagicKind>>;
        "Returns the combatant's damage immunities.",
        damage_immunities, &HashMap<DamageKind, EnumSet<MagicKind>>;
        "Returns the combatant's proficiency bonus.", proficiency_bonus, Modifier;
        "Returns the actions, bonus actions, reactions, and legendary actions the creature can take.",
        actions, Vec<Action>;
    );

    /// Damage the combatant by the given amount, optionally taking its vulnerabilities,
    /// resistances, and immunities into account.
    ///
    /// The amount will not saturate to 0, meaning the combatant can have negative hit points.
    pub fn damage(&mut self, mut amount: i32, kind: Option<DamageKind>) {
        if let Some(kind) = kind {
            // TODO: does not check for magical / nonmagical, should be overridable by user
            let is_immune = self.damage_immunities().contains_key(&kind);
            if is_immune {
                return;
            }

            let is_vulnerable = self.damage_vulnerabilities().contains_key(&kind);
            let is_resistant = self.damage_resistances().contains_key(&kind);
            match (is_vulnerable, is_resistant) {
                (true, false) => amount *= 2,
                (false, true) => amount /= 2,
                _ => (), // cancel the effect(s)
            }
        }
        self.hit_points -= amount;
    }
}

/// The kind of creature the combatant is.
#[derive(Debug)]
pub enum CombatantKind {
    /// A player-controlled character.
    Character(Character),

    /// Pre-made monster.
    Monster(Monster),
}

impl From<Character> for CombatantKind {
    fn from(character: Character) -> Self {
        Self::Character(character)
    }
}

impl From<Character> for Combatant {
    fn from(character: Character) -> Self {
        Self {
            initiative: None,
            group: String::new(),
            hit_points: character.hit_points,
            conditions: Vec::new(),
            kind: character.into(),
            resource_pool: ResourcePool::default(),
        }
    }
}

impl From<Monster> for CombatantKind {
    fn from(monster: Monster) -> Self {
        Self::Monster(monster)
    }
}

impl From<Monster> for Combatant {
    fn from(monster: Monster) -> Self {
        Self {
            initiative: None,
            group: String::new(),
            hit_points: monster.hit_points,
            conditions: Vec::new(),
            kind: monster.into(),
            resource_pool: ResourcePool::default(),
        }
    }
}

/// Makes trivial `impl`s of common getter functions on [`CombatantKind`]s.
macro_rules! combatant_kind_impls {
    ($($doc:literal, $fn_name:ident, |$in:ident| $out:expr, $return_type:ty);+ $(;)?) => {
        $(
            #[doc = $doc]
            pub fn $fn_name(&self) -> $return_type {
                match &self {
                    CombatantKind::Character($in) => $out,
                    CombatantKind::Monster($in) => $out,
                }
            }
        )+
    }
}

impl CombatantKind {
    combatant_kind_impls!(
        "Returns the combatant's name.", name, |c| &c.name, &str;
        "Returns the combatant's ability scores.", scores, |c| c.scores, Ability<Score>;
        "Returns the combatant's proficiencies.", proficiencies, |c| c.proficiencies, Proficiencies;
        "Returns the combatant's main armor class.", armor_class, |c| c.armor_class.value, u32;
        "Returns the combatant's main base speed.", speed, |c| &c.speed, &Speed;
        "Returns the combatant's maximum hit points.", max_hit_points, |c| c.hit_points, i32;
        "Returns the combatant's damage vulnerabilities.",
        damage_vulnerabilities, |c| &c.damage_vulnerabilities, &HashMap<DamageKind, EnumSet<MagicKind>>;
        "Returns the combatant's damage resistances.",
        damage_resistances, |c| &c.damage_resistances, &HashMap<DamageKind, EnumSet<MagicKind>>;
        "Returns the combatant's damage immunities.",
        damage_immunities, |c| &c.damage_immunities, &HashMap<DamageKind, EnumSet<MagicKind>>;
        "Returns the combatant's proficiency bonus.",
        proficiency_bonus, |c| c.proficiency_bonus(), Modifier;
        "Returns the actions, bonus actions, reactions, and legendary actions the creature can take.",
        actions, |c| c.actions(), Vec<Action>;
    );
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
        // advance condition durations
        self.current_combatant_mut()
            .conditions
            .retain_mut(|c| {
                let new_duration = c.duration.decrement();
                if let Some(new) = new_duration {
                    c.duration = new;
                    true
                } else {
                    // condition expired
                    false
                }
            });

        self.turn = (self.turn + 1) % self.combatants.len();
        if self.turn == 0 {
            self.round += 1;
        }

        // restore current combatant's actions at the start of their turn
        // TODO: will reset class and spell things when they shouldn't be reset
        self.current_combatant_mut().resource_pool = ResourcePool::default();
    }

    /// Get the combatant that is currently taking their turn.
    pub fn current_combatant(&self) -> &Combatant {
        &self.combatants[self.turn]
    }

    /// Get mutable access to the combatant that is currently taking their turn.
    pub fn current_combatant_mut(&mut self) -> &mut Combatant {
        &mut self.combatants[self.turn]
    }

    /// Use an action for the current combatant. Returns `true` if the action was used, or `false`
    /// if the combatant had no actions left to use.
    ///
    /// This function only decrements the number of actions available to the combatant, meaning the
    /// combat log will not display any information about the action taken.
    pub fn use_action(&mut self, action: &Action) -> bool {
        let combatant = self.current_combatant_mut();

        for cost in action.costs() {
            let current_count = combatant.resource_pool.get_mut(&cost.resource);
            // TODO: disallow if costs requirements can't be met
            *current_count = current_count.saturating_sub(cost.amount as i32);
        }

        for effect in action.trigger_effects() {
            effect.apply_to_pool(&mut combatant.resource_pool);
        }

        true
    }

    /// Use a bonus action for the current combatant. Returns `true` if the bonus action was used,
    /// or `false` if the combatant had no bonus actions left to use.
    ///
    /// This function only decrements the number of bonus actions available to the combatant,
    /// meaning the combat log will not display any information about the bonus action taken.
    pub fn use_bonus_action(&mut self) -> bool {
        let count = BonusAction::get_mut(&mut self.combatants[self.turn].resource_pool);
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
        let count = Reaction::get_mut(&mut self.combatants[self.turn].resource_pool);
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
