use serde::{Deserialize, Deserializer, Serialize};

/// The classes a (typically) player character can have.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClassKind {
    Barbarian,
    Bard,
    Cleric,
    Druid,
    Fighter,
    Monk,
    Paladin,
    Ranger,
    Rogue,
    Sorcerer,
    Warlock,
    Wizard,
}

pub(crate) fn deserialize_class_kinds<'de, D>(deserializer: D) -> Result<Vec<ClassKind>, D::Error>
where D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct ClassData {
        index: ClassKind,
    }

    let data = <Vec<ClassData>>::deserialize(deserializer)?;
    Ok(data.into_iter()
        .map(|d| d.index)
        .collect())
}
