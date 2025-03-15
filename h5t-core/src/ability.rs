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
    pub strength: T,
    pub dexterity: T,
    pub constitution: T,
    pub intelligence: T,
    pub wisdom: T,
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
