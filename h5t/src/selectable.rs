use h5t_core::{ConditionKind, DamageKind};
use std::{fmt::Display, hash::Hash};

/// Marker trait for `enum`s that can enumerate a number of options that can be selected.
pub(crate) trait SelectableEnum: Copy + Hash + Eq + Display {
    /// Returns a slice of the possible options of the type.
    fn variants() -> &'static [Self];

    /// Returns an iterator over the possible options of the type. This is meant as a convenience
    /// for if the type to iterate over is known at compile time.
    fn owned_variants() -> impl IntoIterator<Item = Self> where Self: 'static {
        Self::variants()
            .iter()
            .copied()
    }
}

impl SelectableEnum for DamageKind {
    fn variants() -> &'static [Self] {
        &[
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
        ]
    }
}

impl SelectableEnum for ConditionKind {
    fn variants() -> &'static [Self] {
        &[
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
        ]
    }
}
