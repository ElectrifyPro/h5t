use h5t_core::Condition;
use std::{fmt::Display, hash::Hash};

/// Marker type for enums that can be used with [`Tracker::multi_select_enum`].
pub(crate) trait Selectable: Copy + Hash + Eq + Display {
    /// The number of variants in the enum.
    const N: usize;

    /// Returns the possible variants of the enum.
    fn variants() -> impl Iterator<Item = Self>;
}

impl Selectable for Condition {
    const N: usize = 15;

    fn variants() -> impl Iterator<Item = Self> {
        [
            Condition::Blinded,
            Condition::Charmed,
            Condition::Deafened,
            Condition::Exhaustion,
            Condition::Frightened,
            Condition::Grappled,
            Condition::Incapacitated,
            Condition::Invisible,
            Condition::Paralyzed,
            Condition::Petrified,
            Condition::Poisoned,
            Condition::Prone,
            Condition::Restrained,
            Condition::Stunned,
            Condition::Unconscious,
        ].into_iter()
    }
}
