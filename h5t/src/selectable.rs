use h5t_core::Condition;
use std::{fmt::Display, hash::Hash};

/// Marker type for enums that can be used with [`Tracker::multi_select_enum`].
pub trait Selectable<const N: usize>: Copy + Hash + Eq + Display {
    /// Returns the possible variants of the enum.
    fn variants() -> [Self; N];
}

impl Selectable<15> for Condition {
    fn variants() -> [Self; 15] {
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
        ]
    }
}
