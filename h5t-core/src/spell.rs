use crate::{class::deserialize_class_kinds, ClassKind, DamageKind};
use serde::{de::Error, Serialize, Deserialize, Deserializer};
use std::num::NonZeroU32;

/// A school of magic a spell can belong to.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum School {
    /// A spell that prevents or reverses harmful effects.
    Abjuration,

    /// A spell that transports creatures or objects.
    Conjuration,

    /// A spell that reveals information.
    Divination,

    /// A spell that influences minds.
    Enchantment,

    /// A spell that channels energy to create effects that are often destructive.
    Evocation,

    /// A spell that deceives the mind or senses.
    Illusion,

    /// A spell that manipulates life and death.
    Necromancy,

    /// A spell that transforms creatures or objects.
    Transmutation,
}

fn deserialize_school<'de, D>(deserializer: D) -> Result<School, D::Error>
where D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct SchoolData {
        index: School,
    }

    let data = SchoolData::deserialize(deserializer)?;
    Ok(data.index)
}

/// Components required to cast a spell.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Components {
    /// The creature must be able to verbalize an incantation to cast the spell.
    verbal: bool,

    /// The creature must be able to gesticulate to cast the spell.
    somatic: bool,

    /// The creature must have the specified material (or a component pouch / spellcasting
    /// focus) to cast the spell.
    material: Option<String>,
}

fn extract_components(raw_components: Vec<String>, mut raw_material: Option<String>) -> Components {
    let mut result = Components::default();

    for component in raw_components {
        match &*component {
            "V" => result.verbal = true,
            "S" => result.somatic = true,
            "M" if result.material.is_none() => result.material = raw_material.take(),
            _ => (),
        }
    }

    result
}

/// The amount of time / number of actions the spellcaster must spend to cast a spell.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CastingTime {
    /// The spell takes an action to cast.
    Action,

    /// The spell takes a bonus action to cast.
    BonusAction,

    /// The spell takes a reaction to cast.
    Reaction,

    /// The spell takes the given number of minutes to cast.
    Minutes(NonZeroU32),

    /// The spell takes the given number of hours to cast.
    Hours(NonZeroU32),
}

fn deserialize_casting_time<'de, D>(deserializer: D) -> Result<CastingTime, D::Error>
where D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let result = match &*s {
        "1 action" => CastingTime::Action,
        "1 bonus action" => CastingTime::BonusAction,
        "1 reaction" => CastingTime::BonusAction,
        other => {
            let mut iter = other.split(' ');

            // same strategy as `Duration` below

            let (maybe_unit, maybe_value) = (
                iter.next_back().ok_or(Error::custom("expected a duration unit or resource"))?,
                iter.next_back().ok_or(Error::custom("expected a numeric value"))?,
            );
            let value = maybe_value.parse().map_err(|_| Error::custom("expected a numeric value"))?;

            match maybe_unit {
                "minute" | "minutes" => CastingTime::Minutes(value),
                "hour" | "hours" => CastingTime::Hours(value),
                _ => return Err(Error::custom("expected a duration unit")),
            }
        },
    };

    Ok(result)
}

/// The period of time a spell is in effect for.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Duration {
    /// The spell takes effect immediately and ends immediately.
    Instantaneous,

    /// The spell lasts the given number of rounds.
    Rounds(NonZeroU32),

    /// The spell lasts the given number of minutes.
    Minutes(NonZeroU32),

    /// The spell lasts the given number of hours.
    Hours(NonZeroU32),

    /// The spell lasts the given number of days.
    Days(NonZeroU32),

    /// The spell's effects last forever unless dispelled.
    UntilDispelled,

    /// The spell has a more complex duration specified in its description.
    Special,
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let result = match &*s {
        "Instantaneous" => Duration::Instantaneous,
        "Until dispelled" => Duration::UntilDispelled,
        "Special" => Duration::Special,
        other => {
            let mut iter = other.split(' ');

            // durations include
            // - "10 minutes"
            // - "1 minute"
            // - "1 hour"
            // - "Up to 1 hour"
            //
            // if we just look at the last two words, we will get a duration and unit

            let (maybe_unit, maybe_value) = (
                iter.next_back().ok_or(Error::custom("expected a duration unit"))?,
                iter.next_back().ok_or(Error::custom("expected a numeric value"))?,
            );
            let value = maybe_value.parse().map_err(|_| Error::custom("expected a numeric value"))?;

            match maybe_unit {
                "round" | "rounds" => Duration::Rounds(value),
                "minute" | "minutes" => Duration::Minutes(value),
                "hour" | "hours" => Duration::Hours(value),
                "day" | "days" => Duration::Days(value),
                _ => return Err(Error::custom("expected a duration unit")),
            }
        },
    };

    Ok(result)
}

/// Any damage a spell deals.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Damage {
    #[serde(rename = "damage_type", deserialize_with = "deserialize_damage_kind")]
    kind: DamageKind,
}

fn deserialize_damage_kind<'de, D>(deserializer: D) -> Result<DamageKind, D::Error>
where D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct DamageData {
        index: DamageKind,
    }

    let data = DamageData::deserialize(deserializer)?;
    Ok(data.index)
}

/// Raw spell schema from the API.
///
/// This helps merge the `components` and `material` fields in the schema into one.
#[derive(Deserialize)]
// #[serde(deny_unknown_fields)] // TODO: missing `subclasses`
// TODO: spells that didn't fit schema and had to be manually modified (added damage_type.index =
// 'acid') to work: Prismatic Spray (multiple damage kinds), Sleep (has a `damage` field but does
// not do damage)
struct RawSpell {
    index: String,
    name: String,
    level: u8,
    #[serde(deserialize_with = "deserialize_school")]
    school: School,
    range: String,
    components: Vec<String>,
    #[serde(deserialize_with = "deserialize_casting_time")]
    casting_time: CastingTime,
    ritual: bool,
    #[serde(deserialize_with = "deserialize_duration")]
    duration: Duration,
    concentration: bool,
    material: Option<String>,
    attack_type: Option<String>,
    damage: Option<Damage>,
    desc: Vec<String>,
    higher_level: Vec<String>,
    #[serde(deserialize_with = "deserialize_class_kinds")]
    classes: Vec<ClassKind>,
}

impl From<RawSpell> for Spell {
    fn from(raw: RawSpell) -> Self {
        let RawSpell {
            index,
            name,
            level,
            school,
            range,
            components,
            casting_time,
            ritual,
            duration,
            concentration,
            material,
            attack_type,
            damage,
            desc,
            higher_level,
            classes,
        } = raw;
        Spell {
            index,
            name,
            level,
            school,
            range,
            components: extract_components(components, material),
            casting_time,
            ritual,
            duration,
            concentration,
            attack_type,
            damage,
            desc,
            higher_level,
            classes,
        }
    }
}

/// A spell that can be cast.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(from = "RawSpell")]
pub struct Spell {
    /// The spell's index, used for identification.
    pub index: String,

    /// The spell's name.
    pub name: String,

    /// The spell's level. Cantrips are level 0.
    pub level: u8,

    /// The school of magic this spell belongs to.
    #[serde(deserialize_with = "deserialize_school")]
    pub school: School,

    /// The range from the spellcaster at which the spell can be cast.
    pub range: String,

    /// The components needed to cast the spell.
    pub components: Components,

    /// The amount of time / number of actions the spellcaster must spend to cast the spell.
    pub casting_time: CastingTime,

    /// Whether the spell is a ritual spell. A ritual spell can be cast without a spell slot at
    /// the cost of not being upcastable, and taking 10 more minutes.
    pub ritual: bool,

    /// The period of time the spell is in effect for.
    pub duration: Duration,

    /// Whether the spell requires concentration.
    pub concentration: bool,

    /// Whether the spell is a melee or ranged attack.
    attack_type: Option<String>,

    /// Any damage a spell deals.
    damage: Option<Damage>,

    /// The spell's long description, broken up into each paragraph.
    pub desc: Vec<String>,

    /// The spell's long description for its upcasting effects, broken up into each paragraph.
    pub higher_level: Vec<String>,

    /// Spellcasting classes that typically can learn this spell.
    pub classes: Vec<ClassKind>,
}
