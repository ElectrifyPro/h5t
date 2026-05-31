use crate::{
    ability::{Modifier, Proficiencies, Score},
    monster::ArmorClass,
    Ability,
    Action,
    DamageKind,
    MagicKind,
    Speed,
};
use enumset::EnumSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A (typically) player-controlled character.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Character {
    /// The character's index, used for identification.
    pub index: String,

    /// The character's name.
    pub name: String,

    /// The character's ability scores, used for calculating modifiers.
    pub scores: Ability<Score>,

    /// The character's armor class, the amount needed to hit them an attack.
    pub armor_class: ArmorClass,

    /// The character's base hit points.
    pub hit_points: i32,

    /// Types of damage the character is vulnerable to. Damage dealt to the character of any of
    /// these types will be doubled.
    pub damage_vulnerabilities: HashMap<DamageKind, EnumSet<MagicKind>>,

    /// Types of damage the character is resistant to. Damage dealt to the character of any of these
    /// types will be halved (rounded down).
    pub damage_resistances: HashMap<DamageKind, EnumSet<MagicKind>>,

    /// Types of damage the character is immune to. Damage dealt to the character of any of these
    /// types will be entirely negated.
    pub damage_immunities: HashMap<DamageKind, EnumSet<MagicKind>>,

    /// The different speeds the character has, such as walking, flying, or swimming.
    pub speed: Speed,

    /// The character's proficiencies, including their skill and saving throw proficiencies.
    pub proficiencies: Proficiencies,

    /// The character's level, determining their proficiency bonus and class features.
    pub level: u8,
}

impl Character {
    /// Returns the actions, bonus actions, reactions, and legendary actions the character can take.
    pub fn actions(&self) -> Vec<Action> {
        Action::standard_actions()
    }

    /// Returns the character's proficiency bonus based on their level.
    pub fn proficiency_bonus(&self) -> Modifier {
        level_to_proficiency(self.level)
    }
}

/// Computes the proficiency given the creature level.
pub(crate) fn level_to_proficiency(level: u8) -> Modifier {
    (level.saturating_sub(1) / 4 + 2).into()
}

#[cfg(test)]
mod tests {
    use super::level_to_proficiency;

    /// Ensure the proficiency bonus calcuation is correct.
    #[test]
    fn modifier_calculation() {
        // see: https://www.dndbeyond.com/sources/dnd/basic-rules-2014/step-by-step-characters#CharacterAdvancement
        let level_to_bonus = [
            (1, 2),
            (2, 2),
            (3, 2),
            (4, 2),
            (5, 3),
            (6, 3),
            (7, 3),
            (8, 3),
            (9, 4),
            (10, 4),
            (11, 4),
            (12, 4),
            (13, 5),
            (14, 5),
            (15, 5),
            (16, 5),
            (17, 6),
            (18, 6),
            (19, 6),
            (20, 6),
        ];

        for (level, bonus_modifier) in level_to_bonus.iter() {
            assert_eq!(level_to_proficiency(*level), *bonus_modifier);
        }
    }
}
