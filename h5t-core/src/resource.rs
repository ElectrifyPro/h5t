//! Resources that a creature can spend to perform actions.

use crate::Id;
use std::{borrow::Cow, collections::HashMap};

/// The number of resources available to a combatant, including action count, bonus action count,
/// reaction count, movement, and resources granted by classes (e.g. Superiority dice) and spells
/// (e.g. Haste action).
#[derive(Clone, Debug)]
pub struct ResourcePool(pub(crate) HashMap<Id, i32>);

impl ResourcePool {
    /// Get the count for a specified resource.
    pub fn get(&self, id: &Id) -> i32 {
        self.0.get(id).copied().unwrap_or(0)
    }

    /// Get a mutable reference to the count for a specified resource.
    pub fn get_mut(&mut self, id: &Id) -> &mut i32 {
        self.0.entry(id.clone()).or_default()
    }

    /// Determines if there are enough resources in this pool to perform an action that costs the
    /// given amount.
    pub fn can_perform(&self, costs: &[Cost]) -> bool {
        let mut required = HashMap::new();
        for cost in costs {
            *required.entry(cost.resource.clone()).or_default() += cost.amount;
        }

        required.into_iter().all(|(resource, needed)| {
            let available = self.get(&resource);

            if available < 0 {
                return false;
            }

            available as u32 >= needed
        })
    }
}

/// By default, creatures have one action, bonus action, and reaction per turn.
impl Default for ResourcePool {
    fn default() -> Self {
        Self(HashMap::from([
            (Action::ID, 1),
            (BonusAction::ID, 1),
            (Reaction::ID, 1),
        ]))
    }
}

/// Trait for marker types that identify resources in the [`ResourcePool`].
pub trait Resource {
    const ID: Id;

    fn get(pool: &ResourcePool) -> i32 {
        pool.get(&Self::ID)
    }

    fn get_mut(pool: &mut ResourcePool) -> &mut i32 {
        pool.get_mut(&Self::ID)
    }
}

/// Number of actions, used for attacks, casting various spells, helping, etc.
pub struct Action;

impl Resource for Action {
    const ID: Id = Id(Cow::Borrowed("action"));
}

/// Number of bonus actions, used for off-hand attacks, casting certain spells, some class actions,
/// etc.
pub struct BonusAction;

impl Resource for BonusAction {
    const ID: Id = Id(Cow::Borrowed("bonus-action"));
}

/// Number of reactions, used for opportunity attacks, Counterspell, etc.
pub struct Reaction;

impl Resource for Reaction {
    const ID: Id = Id(Cow::Borrowed("reaction"));
}

/// The amount of a resource needed to perform something.
#[derive(Clone, Debug)]
pub struct Cost {
    pub resource: Id,
    pub amount: u32,
}
