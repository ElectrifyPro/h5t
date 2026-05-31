use serde::{Deserialize, Serialize};

/// An ability score (1-30).
pub type Score = i32;

/// An ability modifier (-5 to +10).
pub type Modifier = i32;

/// Computes the modifier for an ability score.
pub fn score_to_modifier(score: Score) -> Modifier {
    // NOTE: the mathematically equivalent: (score - 10) / 2
    // does not work since integer division will truncate the result, causing scores less than
    // 10 to get rounded up instead of down; so we subtract at the end instead
    score / 2 - 5
}

/// A type that packs together all six ability values.
///
/// It can represent the ability scores themselves, the ability score modifiers, or any other
/// numerical values related to abilities, depending on the parameter chosen for the type `T`.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Ability<T> {
    /// Natural athleticism, bodily power, and physical might.
    pub strength: T,

    /// Physical agility, reflexes, and balance.
    pub dexterity: T,

    /// Health, stamina, and endurance.
    pub constitution: T,

    /// Reasoning, memory, and mental acuity.
    pub intelligence: T,

    /// Awareness, intuition, insight, and mental fortitude.
    pub wisdom: T,

    /// Confidence, eloquence, leadership, and charm.
    pub charisma: T,
}

impl Ability<Score> {
    /// Convert the ability scores to ability modifiers.
    pub fn modifiers(&self) -> Ability<Modifier> {
        Ability {
            strength: score_to_modifier(self.strength),
            dexterity: score_to_modifier(self.dexterity),
            constitution: score_to_modifier(self.constitution),
            intelligence: score_to_modifier(self.intelligence),
            wisdom: score_to_modifier(self.wisdom),
            charisma: score_to_modifier(self.charisma),
        }
    }
}

/// A type that packs together all skills.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Skill<T> {
    /// Acrobatics (Dexterity).
    pub acrobatics: T,

    /// Animal Handling (Wisdom).
    pub animal_handling: T,

    /// Arcana (Intelligence).
    pub arcana: T,

    /// Athletics (Strength).
    pub athletics: T,

    /// Deception (Charisma).
    pub deception: T,

    /// History (Intelligence).
    pub history: T,

    /// Insight (Wisdom).
    pub insight: T,

    /// Intimidation (Charisma).
    pub intimidation: T,

    /// Investigation (Intelligence).
    pub investigation: T,

    /// Medicine (Wisdom).
    pub medicine: T,

    /// Nature (Intelligence).
    pub nature: T,

    /// Perception (Wisdom).
    pub perception: T,

    /// Performance (Charisma).
    pub performance: T,

    /// Persuasion (Charisma).
    pub persuasion: T,

    /// Religion (Intelligence).
    pub religion: T,

    /// Sleight of Hand (Dexterity).
    pub sleight_of_hand: T,

    /// Stealth (Dexterity).
    pub stealth: T,

    /// Survival (Wisdom).
    pub survival: T,
}

/// A creature's proficiencies.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Proficiencies {
    /// The creature's skill proficiencies.
    ///
    /// If the creature has proficiency in a skill, its modifier will be `Some`, and will contain
    /// its proficiency bonus plus its ability modifier for the relevant ability score. Otherwise,
    /// the value will be `None`, and the creature will use the ability modifier alone to calculate
    /// the skill check.
    #[serde(default)]
    pub skills: Skill<Option<Modifier>>,

    /// The creature's saving throw proficiencies.
    ///
    /// If the creature has proficiency in a saving throw, its modifier will be `Some`, and will
    /// contain its proficiency bonus plus its ability modifier for the relevant ability score.
    /// Otherwise, the value will be `None`, and the creature will use the ability modifier alone to
    /// calculate the saving throw.
    #[serde(default)]
    pub saving_throws: Ability<Option<Modifier>>,
}

#[cfg(test)]
mod tests {
    use super::score_to_modifier;

    /// Ensure the modifier calculation is correct.
    #[test]
    fn modifier_calculation() {
        let tests = [
            (1, -5),
            (2, -4),
            (3, -4),
            (4, -3),
            (5, -3),
            (6, -2),
            (7, -2),
            (8, -1),
            (9, -1),
            (10, 0),
            (11, 0),
            (12, 1),
            (13, 1),
            (14, 2),
            (15, 2),
            (16, 3),
            (17, 3),
            (18, 4),
            (19, 4),
            (20, 5),
            (21, 5),
            (22, 6),
            (23, 6),
            (24, 7),
            (25, 7),
            (26, 8),
            (27, 8),
            (28, 9),
            (29, 9),
            (30, 10),
        ];

        for (score, modifier) in tests.iter() {
            assert_eq!(score_to_modifier(*score), *modifier);
        }
    }
}
