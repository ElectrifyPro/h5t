use crate::{
    resource::{
        Action as ActionRes,
        BonusAction,
        Cost,
        Reaction,
        Resource,
        ResourcePool,
        SpeedMultiplier,
    },
    Id,
};

#[derive(Clone, Debug)]
pub enum Effect {
    /// Add a fixed amount of a resource identified by [`Id`] to the character's resource pool.
    GrantResource(Id, u32),
}

impl Effect {
    pub fn apply_to_pool(&self, pool: &mut ResourcePool) {
        match self {
            Effect::GrantResource(id, amount) => {
                let amount = *amount as i32;
                pool.0.entry(id.clone())
                    .and_modify(|count| *count += amount)
                    .or_insert(amount);
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Action {
    /// Unique identifier for the action.
    id: String,

    pub name: String,

    /// Full-length description of the action.
    desc: String,

    /// Resources to spend to take the action.
    costs: Vec<Cost>,

    /// Effects that happen when the action is taken.
    on_trigger: Vec<Effect>,
}

impl Action {
    pub fn costs(&self) -> &[Cost] {
        &self.costs
    }

    pub fn trigger_effects(&self) -> &[Effect] {
        &self.on_trigger
    }

    pub fn standard_actions() -> Vec<Action> {
        vec![
            Action {
                id: "dash".to_string(),
                name: "Dash".to_string(),
                desc: "When you take the Dash action, you gain extra movement for the current turn. The increase equals your speed, after applying any modifiers. With a speed of 30 feet, for example, you can move up to 60 feet on your turn if you dash.

Any increase or decrease to your speed changes this additional movement by the same amount. If your speed of 30 feet is reduced to 15 feet, for instance, you can move up to 30 feet this turn if you dash.".to_string(),
                costs: vec![
                    Cost {
                        resource: ActionRes::ID,
                        amount: 1,
                    },
                ],
                on_trigger: vec![
                    Effect::GrantResource(SpeedMultiplier::ID, 1),
                ],
            },
            Action {
                id: "convert-action".to_string(),
                name: "Convert Action".to_string(),
                desc: "testing: convert your action to a bonus action".to_string(),
                costs: vec![
                    Cost {
                        resource: ActionRes::ID,
                        amount: 1,
                    },
                ],
                on_trigger: vec![
                    Effect::GrantResource(BonusAction::ID, 1),
                ],
            },
            Action {
                id: "free-action+reactions".to_string(),
                name: "Free Action and Reactions".to_string(),
                desc: "testing: get a free action and 2 reactions".to_string(),
                costs: vec![],
                on_trigger: vec![
                    Effect::GrantResource(ActionRes::ID, 1),
                    Effect::GrantResource(Reaction::ID, 2),
                ],
            },
        ]
    }
}

// /// The standard actions any creature can spend an action performing.
// pub enum StandardAction {
//     /// Attack using a carried weapon or your fists.
//     Attack,
//
//     /// Cast a spell from memory or through a scroll. Note that not all spells cost an action to
//     /// perform.
//     Cast,
//
//     /// Gain additional movement speed for your turn.
//     Dash,
//
//     /// Prevent opportunity attacks from being triggered by your movement.
//     Disengage,
//
//     /// Impose disadvantage on attack rolls against you, and gain advantage on Dexterity saving
//     /// throws.
//     Dodge,
//
//     /// Help a creature perform a task.
//     Help,
//
//     /// Hide from creatures.
//     Hide,
//
//     /// Use your reaction to act at a more favorable moment.
//     Ready,
//
//     /// Search for something.
//     Search,
//
//     /// Interact with an object.
//     Use,
//
//     Improvise,
// }
