use crate::{
    ability::{Modifier, Score, Skill},
    damage::deserialize_damage_kinds,
    Ability,
    Action,
    DamageKind,
    MagicKind,
};
use enumset::EnumSet;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// The source of a monster's armor class value.
#[derive(Clone, Debug, Default, Serialize)]
pub enum ArmorClassSource {
    /// The armor class is calculated from the monster's Dexterity modifier (i.e., 10 + DEX mod).
    #[default]
    Dexterity,

    /// The monster has natural armor that provides a fixed armor class value.
    Natural,

    /// The monster has armor that provides a fixed armor class value.
    Armor,
}

/// A monster's armor class.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ArmorClass {
    /// The source of the armor class value.
    pub source: ArmorClassSource,

    /// The armor class value.
    pub value: u32,
}

fn deserialize_armor_class<'de, D>(d: D) -> Result<ArmorClass, D::Error>
where D: Deserializer<'de>
{
    // api provides an array of:
    //
    // {"type": "natural" | "dex", "value": 12}
    //
    // look for the first and use it

    #[derive(Debug, Deserialize)]
    struct AcData {
        r#type: String,
        value: u32,
    }

    let data = Vec::<AcData>::deserialize(d)?;
    data.into_iter()
        .find_map(|data| {
            let source = match data.r#type.as_str() {
                "dex" => ArmorClassSource::Dexterity,
                "natural" => ArmorClassSource::Natural,
                "armor" => ArmorClassSource::Armor,
                _ => return None,
            };
            Some(ArmorClass { source, value: data.value })
        })
        .ok_or_else(|| serde::de::Error::custom("invalid armor class data"))
}

/// A creature's speed (in feet) on all types of movement.
///
/// Each field is given as a descriptive string, such as "30 ft.".
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Speed {
    /// Basic movement speed.
    #[serde(default, deserialize_with = "deserialize_inner_speed")]
    pub walk: Option<i32>,

    /// Movement speed when moving through sand, earth, mud, or ice.
    #[serde(default, deserialize_with = "deserialize_inner_speed")]
    pub burrow: Option<i32>,

    /// Movement speed when climbing.
    #[serde(default, deserialize_with = "deserialize_inner_speed")]
    pub climb: Option<i32>,

    /// Movement speed when flying.
    #[serde(default, deserialize_with = "deserialize_inner_speed")]
    pub fly: Option<i32>,

    /// Movement speed when swimming.
    #[serde(default, deserialize_with = "deserialize_inner_speed")]
    pub swim: Option<i32>,
}

fn deserialize_inner_speed<'de, D>(d: D) -> Result<Option<i32>, D::Error>
where D: Deserializer<'de>
{
    // api provides speed as: {"walk":"10 ft.","swim":"40 ft."}
    //
    // pretty easy to get what we want
    let result = String::deserialize(d)?
        .split(' ')
        .next()
        .map(|s| s.parse().ok())
        .flatten();
    Ok(result)
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

/// A monster's proficiencies.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Proficiencies {
    /// The monster's skill proficiencies.
    ///
    /// If the monster has proficiency in a skill, its modifier will be `Some`, and will contain
    /// its proficiency bonus plus its ability modifier for the relevant ability score. Otherwise,
    /// the value will be `None`, and the monster will use the ability modifier alone to calculate
    /// the skill check.
    #[serde(default)]
    pub skills: Skill<Option<Modifier>>,

    /// The monster's saving throw proficiencies.
    ///
    /// If the monster has proficiency in a saving throw, its modifier will be `Some`, and will
    /// contain its proficiency bonus plus its ability modifier for the relevant ability score.
    /// Otherwise, the value will be `None`, and the monster will use the ability modifier alone to
    /// calculate the saving throw.
    #[serde(default)]
    pub saving_throws: Ability<Option<Modifier>>,
}

fn deserialize_proficiencies<'de, D>(d: D) -> Result<Proficiencies, D::Error>
where D: Deserializer<'de>
{
    // api provides one array of modifiers for both skills and saving throws, they contain things
    // like
    //
    // {"value": 11, "proficiency": {"index": "saving-throw-wis", ...}}

    #[derive(Debug, Deserialize)]
    struct ProfDataInner {
        index: String,
    }

    #[derive(Debug, Deserialize)]
    struct ProfData {
        value: u32,
        proficiency: ProfDataInner,
    }

    let mut proficiencies = Proficiencies::default();
    let data = Vec::<ProfData>::deserialize(d)?;
    for prof in data {
        let modifier = Some(prof.value as i32);
        match prof.proficiency.index.as_str() {
            "saving-throw-str" => proficiencies.saving_throws.strength = modifier,
            "saving-throw-dex" => proficiencies.saving_throws.dexterity = modifier,
            "saving-throw-con" => proficiencies.saving_throws.constitution = modifier,
            "saving-throw-int" => proficiencies.saving_throws.intelligence = modifier,
            "saving-throw-wis" => proficiencies.saving_throws.wisdom = modifier,
            "saving-throw-cha" => proficiencies.saving_throws.charisma = modifier,
            "skill-acrobatics" => proficiencies.skills.acrobatics = modifier,
            "skill-animal-handling" => proficiencies.skills.animal_handling = modifier,
            "skill-arcana" => proficiencies.skills.arcana = modifier,
            "skill-athletics" => proficiencies.skills.athletics = modifier,
            "skill-deception" => proficiencies.skills.deception = modifier,
            "skill-history" => proficiencies.skills.history = modifier,
            "skill-insight" => proficiencies.skills.insight = modifier,
            "skill-intimidation" => proficiencies.skills.intimidation = modifier,
            "skill-investigation" => proficiencies.skills.investigation = modifier,
            "skill-medicine" => proficiencies.skills.medicine = modifier,
            "skill-nature" => proficiencies.skills.nature = modifier,
            "skill-perception" => proficiencies.skills.perception = modifier,
            "skill-performance" => proficiencies.skills.performance = modifier,
            "skill-persuasion" => proficiencies.skills.persuasion = modifier,
            "skill-religion" => proficiencies.skills.religion = modifier,
            "skill-sleight-of-hand" => proficiencies.skills.sleight_of_hand = modifier,
            "skill-stealth" => proficiencies.skills.stealth = modifier,
            "skill-survival" => proficiencies.skills.survival = modifier,
            _ => (),
        }
    }

    Ok(proficiencies)
}

/// A trait that a monster has.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Trait {
    /// The name of the trait.
    pub name: String,

    /// The description of the trait.
    pub desc: String,

    /// The usage of the trait.
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

/// Usage constraints for a trait.
#[derive(Clone, Debug, Default, Serialize)]
pub enum Usage {
    /// The trait has a limited number of usages per day. Effectively, this is a limit to how many
    /// times the trait can be used in this combat encounter.
    PerDay(usize),

    /// The trait recharges after a short or long rest.
    RechargeAfterRest,

    /// The trait recharges only after a long rest.
    RechargeAfterLongRest,

    /// There is no constraint; the trait can be used at will, or it is a passive ability that is
    /// always active.
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

    /// A string describing the monster's alignment.
    pub alignment: String,

    /// The monster's ability scores, used for calculating modifiers.
    #[serde(flatten)]
    pub scores: Ability<Score>,

    /// The monster's size, used to determine the amount of space it occupies on the battlefield.
    pub size: Size,

    /// The type of monster, used for categorization, or for spells and abilities that target
    /// specific types of creatures.
    pub r#type: Type,

    /// The monster's subtype, if any. This is used for further categorization.
    pub subtype: Option<String>,

    /// The monster's armor class, the amount needed to hit it with an attack.
    #[serde(deserialize_with = "deserialize_armor_class")]
    pub armor_class: ArmorClass,

    /// The monster's hit points.
    pub hit_points: i32,

    /// The expression to roll for the monster's hit points.
    pub hit_points_roll: String,

    /// Types of damage the monster is vulnerable to. Damage dealt to the monster of any of these
    /// types will be doubled.
    #[serde(deserialize_with = "deserialize_damage_kinds")]
    pub damage_vulnerabilities: HashMap<DamageKind, EnumSet<MagicKind>>,

    /// Types of damage the monster is resistant to. Damage dealt to the monster of any of these
    /// types will be halved (rounded down).
    #[serde(deserialize_with = "deserialize_damage_kinds")]
    pub damage_resistances: HashMap<DamageKind, EnumSet<MagicKind>>,

    /// Types of damage the monster is immune to. Damage dealt to the monster of any of these types
    /// will be entirely negated.
    #[serde(deserialize_with = "deserialize_damage_kinds")]
    pub damage_immunities: HashMap<DamageKind, EnumSet<MagicKind>>,

    /// The different speeds the monster has, such as walking, flying, or swimming.
    pub speed: Speed,

    // The monster's proficiencies, including its skill and saving throw proficiencies.
    #[serde(default, deserialize_with = "deserialize_proficiencies")]
    pub proficiencies: Proficiencies,

    /// The monster's challenge rating. Can be `0.0`, `0.125`, `0.25`, `0.5`, or an integer from `1`
    /// to `30`.
    pub challenge_rating: f32,

    /// The XP value of the monster. If the DM is using XP to determine rewards, this is the amount
    /// of XP the party gains for defeating the monster.
    pub xp: i32,

    /// The monster's proficiency bonus, used for calculating attack bonuses and saving throw DCs.
    pub proficiency_bonus: Modifier,

    /// The monster's traits that provide it with various benefits or drawbacks. This includes
    /// things like Legendary Resistances, Lair Actions, etc.
    #[serde(rename = "special_abilities")]
    pub traits: Vec<Trait>,
}

impl Monster {
    /// Returns the actions, bonus actions, reactions, and legendary actions the creature can take.
    pub fn actions(&self) -> Vec<Action> {
        Action::standard_actions()
    }
}
