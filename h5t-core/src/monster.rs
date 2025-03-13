use serde::{Deserialize, Serialize};

/// The ability scores of a creature.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AbilityScores {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

/// A creature's size.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum Size {
    #[default]
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

/// A creature's type.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Aberration,
    Beast,
    Celestial,
    Construct,
    Dragon,
    Elemental,
    Fey,
    Fiend,
    Giant,
    Humanoid,
    Monstrosity,
    Ooze,
    Plant,
    Undead,

    #[default]
    #[serde(other)] // TODO: capture the value of the "other" case
    Other,
}

/// A pre-made monster from the System Reference Document (SRD), or a custom monster.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Monster {
    /// The monster's index, used for identification.
    pub index: String,

    /// The monster's name.
    pub name: String,

    /// The monster's ability scores, used for calculating modifiers.
    #[serde(flatten)]
    pub scores: AbilityScores,

    /// The monster's size, used to determine the amount of space it occupies on the battlefield.
    pub size: Size,

    /// The type of monster, used for categorization, or for spells and abilities that target
    /// specific types of creatures.
    pub r#type: Type,

    /// The monster's armor class, the amount needed to hit it with an attack.
    // pub armor_class: i32,

    /// The monster's hit points.
    pub hit_points: i32,
}
