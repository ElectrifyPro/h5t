use serde::{Deserialize, Deserializer, Serialize};

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

/// A special ability that a monster has.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SpecialAbility {
    /// The name of the special ability.
    pub name: String,

    /// The description of the special ability.
    pub desc: String,

    /// The usage of the special ability.
    #[serde(default, deserialize_with = "deserialize_usage")]
    pub usage: Usage,
}

fn deserialize_usage<'de, D>(d: D) -> Result<Usage, D::Error>
where D: Deserializer<'de>
{
    // api provides either
    //
    // {"type": "per day", "times": 3}
    // or
    // {"type": "recharge after rest", "rest_types": ["short", "long"]}

    #[derive(Debug, Deserialize)]
    struct UsageData {
        times: Option<usize>,
        rest_types: Option<Vec<String>>,
    }

    let data = UsageData::deserialize(d)?;
    match (data.times, data.rest_types) {
        (Some(times), _) if times > 0 => Ok(Usage::PerDay(times)),
        (_, Some(rest_types)) if !rest_types.is_empty() => {
            if rest_types.iter().any(|s| s == "short") {
                // if an ability recharges after a short rest, a long rest covers it too
                Ok(Usage::RechargeAfterRest)
            } else if rest_types.iter().any(|s| s == "long") {
                Ok(Usage::RechargeAfterLongRest)
            } else {
                Err(serde::de::Error::custom("invalid rest types"))
            }
        }
        _ => Err(serde::de::Error::custom("invalid usage data")),
    }
}

/// Usage constraints for a special ability.
#[derive(Clone, Debug, Default, Serialize)]
pub enum Usage {
    /// The special ability has a limited number of usages per day. Effectively, this is a
    /// limit to how many times the special ability can be used in this combat encounter.
    PerDay(usize),

    /// The special ability recharges after a short or long rest.
    RechargeAfterRest,

    /// The special ability recharges only after a long rest.
    RechargeAfterLongRest,

    /// There is no constraint; the special ability can be used at will, or it is a passive
    /// ability that is always active.
    #[default]
    #[serde(other)]
    AtWill,
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

    /// The monster's special abilities that aren't necessarily actions, bonus actions, or
    /// reactions. This includes things like Legendary Resistances, Lair Actions, etc.
    pub special_abilities: Vec<SpecialAbility>,
}
