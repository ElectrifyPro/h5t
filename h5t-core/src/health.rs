use std::num::{NonZeroI32, TryFromIntError};

/// The number of death save successes and failures a combatant has.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeathSaveCount {
    /// The number of death saving throws the combatant has succeeded. When the combatant rolls
    /// three successes, it becomes [`Health::Stabilized`].
    pub successes: u8,

    /// The number of death saving throws the combatant has failed. When the combatant rolls three
    /// failures, it becomes [`Health::Dead`].
    pub failures: u8,
}

impl DeathSaveCount {
    /// Returns a [`DeathSaveCount`] with no successes or failures counted.
    pub fn new() -> Self {
        Self::default()
    }
}

/// The health state of a combatant.
#[derive(Clone, Copy, Debug)]
pub enum Health {
    /// The combatant has the given number of hit points.
    Hp(NonZeroI32),

    /// The combatant is downed. It will begin making death saving throws on each of its turns.
    Downed(DeathSaveCount),

    /// The combatant is stabilized. It will no longer need to make death saving throws, but will
    /// remain unconscious until it is healed.
    Stabilized,

    /// The combatant is dead.
    Dead,
}

impl TryFrom<i32> for Health {
    type Error = TryFromIntError;

    fn try_from(value: i32) -> Result<Health, TryFromIntError> {
        let non_zero = NonZeroI32::try_from(value)?;
        Ok(Self::Hp(non_zero))
    }
}

impl Health {
    /// Returns `true` if the combatant is either [`Health::Hp`] or [`Health::Downed`].
    pub fn active(self) -> bool {
        matches!(self, Health::Hp(_) | Health::Downed { .. })
    }

    /// Returns `true` if the combatant is conscious ([`Health::Hp`]).
    pub fn conscious(self) -> bool {
        matches!(self, Health::Hp(_))
    }
}
