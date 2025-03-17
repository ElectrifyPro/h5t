use enumset::EnumSetType;

/// All possible conditions that can be applied to a combatant.
#[derive(EnumSetType, Debug, Hash)]
pub enum Condition {
    Blinded,
    Charmed,
    Deafened,
    Exhaustion,
    Frightened,
    Grappled,
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Duration of a condition.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ConditionDuration {
    /// The condition lasts until the end of the combatant's next turn.
    #[default]
    UntilNextTurn,

    /// The condition lasts for the given number of rounds.
    ///
    /// When the combatant's turn ends, the duration is decremented by one. When the duration
    /// is reduced to zero, the condition ends.
    Rounds(u32),
}
