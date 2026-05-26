use h5t_core::{ConditionKind, DamageKind};
use std::{fmt::Display, hash::Hash};

/// Marker type for enums that can be used with [`Tracker::multi_select_enum`].
pub(crate) trait Selectable: Copy + Hash + Eq + Display {
    /// The number of variants in the enum.
    const N: usize;

    /// Returns the possible variants of the enum.
    fn variants() -> impl Iterator<Item = Self>;
}

impl Selectable for DamageKind {
    const N: usize = 13;

    fn variants() -> impl Iterator<Item = Self> {
        [
            DamageKind::Acid,
            DamageKind::Bludgeoning,
            DamageKind::Cold,
            DamageKind::Fire,
            DamageKind::Force,
            DamageKind::Lightning,
            DamageKind::Necrotic,
            DamageKind::Piercing,
            DamageKind::Poison,
            DamageKind::Psychic,
            DamageKind::Radiant,
            DamageKind::Slashing,
            DamageKind::Thunder,
        ].into_iter()
    }
}

impl Selectable for ConditionKind {
    const N: usize = 15;

    fn variants() -> impl Iterator<Item = Self> {
        [
            ConditionKind::Blinded,
            ConditionKind::Charmed,
            ConditionKind::Deafened,
            ConditionKind::Exhaustion,
            ConditionKind::Frightened,
            ConditionKind::Grappled,
            ConditionKind::Incapacitated,
            ConditionKind::Invisible,
            ConditionKind::Paralyzed,
            ConditionKind::Petrified,
            ConditionKind::Poisoned,
            ConditionKind::Prone,
            ConditionKind::Restrained,
            ConditionKind::Stunned,
            ConditionKind::Unconscious,
        ].into_iter()
    }
}
