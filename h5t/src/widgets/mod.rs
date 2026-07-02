pub mod ability_scores;
pub mod combatant_block;
pub mod conditions;
pub mod hit_points;
pub mod popup;
pub mod selectable_table;
pub mod setup;
pub mod sized_table;
pub mod stat_block;
pub mod tracker;

pub use ability_scores::AbilityScores;
pub use combatant_block::CombatantBlock;
pub use conditions::CompactConditions;
pub use hit_points::HitPoints;
pub use selectable_table::SelectableTable;
pub use setup::Setup;
pub use sized_table::SizedTable;
pub use stat_block::StatBlock;
pub use tracker::Tracker;

pub(crate) use tracker::max_combatants;

use h5t_core::Speed;

/// Formats a speed value.
///
/// This is used by the [`CombatantBlock`] and [`StatBlock`] widgets.
fn fmt_speed(speed: &Speed) -> String {
    let mut parts = String::new();
    if let Some(speed) = &speed.walk {
        parts.push_str(&speed.to_string());
        parts.push_str(" ft., ");
    }
    if let Some(speed) = &speed.burrow {
        parts.push_str("burrow ");
        parts.push_str(&speed.to_string());
        parts.push_str(" ft., ");
    }
    if let Some(speed) = &speed.climb {
        parts.push_str("climb ");
        parts.push_str(&speed.to_string());
        parts.push_str(" ft., ");
    }
    if let Some(speed) = &speed.fly {
        parts.push_str("fly ");
        parts.push_str(&speed.to_string());
        parts.push_str(" ft., ");
    }
    if let Some(speed) = &speed.swim {
        parts.push_str("swim ");
        parts.push_str(&speed.to_string());
        parts.push_str(" ft., ");
    }
    parts.pop(); // remove trailing space
    parts.pop(); // remove trailing comma
    parts
}
