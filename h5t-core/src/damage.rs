use enumset::{EnumSet, EnumSetType};
use serde::{de::{value::{Error, StrDeserializer}, IntoDeserializer}, Deserialize, Deserializer,  Serialize};
use std::collections::HashMap;

/// Whether damage is magical or nonmagical.
#[derive(EnumSetType, Debug, Hash, Serialize)]
pub enum MagicKind {
    Magical,
    Nonmagical,
}

/// All possible damage types.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DamageKind {
    Acid,
    Bludgeoning,
    Cold,
    Fire,
    Force,
    Lightning,
    Necrotic,
    Piercing,
    Poison,
    Psychic,
    Radiant,
    Slashing,
    Thunder,
}

pub(crate) fn deserialize_damage_kinds<'de, D>(
    d: D,
) -> Result<HashMap<DamageKind, EnumSet<MagicKind>>, D::Error>
where D: Deserializer<'de>,
{
    // api gives a list of strings where each is usually a damage type, but sometimes the string is
    // like: "bludgeoning, piercing, and slashing from nonmagical weapons"

    let descriptions = <Vec<String>>::deserialize(d)?;
    Ok(descriptions
        .into_iter()
        .filter_map(|description| {
            match <DamageKind>::deserialize::<StrDeserializer<Error>>((&*description).into_deserializer()) {
                Ok(out) => Some(vec![(out, EnumSet::all())]), // assume vulnerability / etc. against any kind of attack
                Err(_) => {
                    // TODO: this is crazy, maybe just change the api response
                    if description.contains("bludgeoning, piercing, and slashing from nonmagical weapons") {
                        Some(vec![
                            (DamageKind::Bludgeoning, MagicKind::Nonmagical.into()),
                            (DamageKind::Piercing, MagicKind::Nonmagical.into()),
                            (DamageKind::Slashing, MagicKind::Nonmagical.into()),
                        ])
                    } else if description.contains("damage from spells") {
                        Some(vec![
                            (DamageKind::Acid, MagicKind::Magical.into()),
                            (DamageKind::Bludgeoning, MagicKind::Magical.into()),
                            (DamageKind::Cold, MagicKind::Magical.into()),
                            (DamageKind::Fire, MagicKind::Magical.into()),
                            (DamageKind::Force, MagicKind::Magical.into()),
                            (DamageKind::Lightning, MagicKind::Magical.into()),
                            (DamageKind::Necrotic, MagicKind::Magical.into()),
                            (DamageKind::Piercing, MagicKind::Magical.into()),
                            (DamageKind::Poison, MagicKind::Magical.into()),
                            (DamageKind::Psychic, MagicKind::Magical.into()),
                            (DamageKind::Radiant, MagicKind::Magical.into()),
                            (DamageKind::Slashing, MagicKind::Magical.into()),
                            (DamageKind::Thunder, MagicKind::Magical.into()),
                        ])
                    } else if description.contains("piercing from magic weapons") {
                        Some(vec![(DamageKind::Piercing, MagicKind::Magical.into())])
                    } else if description.contains("piercing and slashing from nonmagical weapons") {
                        Some(vec![
                            (DamageKind::Piercing, MagicKind::Nonmagical.into()),
                            (DamageKind::Slashing, MagicKind::Nonmagical.into()),
                        ])
                    } else {
                        None
                    }
                },
            }
        })
        .flatten()
        .collect())
}

impl std::fmt::Display for DamageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

macro_rules! make_abbreviations {
    ($($kind:ident => $abbreviation:expr),* $(,)?) => {
        impl DamageKind {
            /// Returns an abbreviation for the damage type that identifies it uniquely.
            ///
            /// The current abbreviations used are:
            ///
            $(
                #[doc = concat!(
                    "- [`",
                    stringify!($kind),
                    "`](DamageKind::",
                    stringify!($kind),
                    "): `",
                    $abbreviation,
                    "`\n",
                )]
            )*
            pub fn abbreviation(self) -> &'static str {
                match self {
                    $(DamageKind::$kind => $abbreviation,)*
                }
            }
        }
    };
}

make_abbreviations! {
    Acid => "A",
    Bludgeoning => "B",
    Cold => "C",
    Fire => "F",
    Force => "F",
    Lightning => "L",
    Necrotic => "N",
    Piercing => "PI",
    Poison => "PO",
    Psychic => "PSY",
    Radiant => "R",
    Slashing => "S",
    Thunder => "T",
}
