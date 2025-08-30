use h5t_core::{monster::Speed, Combatant};
use ratatui::{prelude::*, widgets::*};
use crate::{theme::THEME, widgets::conditions::FullConditions};

use super::{AbilityScores, HitPoints};

/// Creates a [`Text`] widget for displaying the combatant's name and whether they are dead.
fn basic_status_text(combatant: &Combatant) -> Text {
    if combatant.hit_points <= 0 {
        Text::styled(format!("{} (Dead)", combatant.name()), Modifier::BOLD)
    } else {
        Text::styled(combatant.name(), Modifier::BOLD)
    }.fg(THEME.foreground)
}

/// Creates a [`Table`] widget for displaying a monster's basic statistics.
fn basic_stats_table(combatant: &Combatant) -> Table {
    /// Format's a speed value.
    fn fmt_speed(speed: &Speed) -> String {
        let mut parts = String::new();
        if let Some(speed) = &speed.walk {
            parts.push_str(speed);
            parts.push_str(", ");
        }
        if let Some(speed) = &speed.burrow {
            parts.push_str("burrow ");
            parts.push_str(speed);
            parts.push_str(", ");
        }
        if let Some(speed) = &speed.climb {
            parts.push_str("climb ");
            parts.push_str(speed);
            parts.push_str(", ");
        }
        if let Some(speed) = &speed.fly {
            parts.push_str("fly ");
            parts.push_str(speed);
            parts.push_str(", ");
        }
        if let Some(speed) = &speed.swim {
            parts.push_str("swim ");
            parts.push_str(speed);
            parts.push_str(", ");
        }
        parts.pop(); // remove trailing comma
        parts.pop(); // remove trailing space
        parts
    }

    Table::new(
        vec![
            Row::new(vec![
                Text::styled("Armor Class", Modifier::BOLD),
                Text::raw(combatant.armor_class().to_string()),
            ]),
            Row::new(vec![
                Text::styled("Hit Points", Modifier::BOLD),
                HitPoints::new(combatant).line().into(),
            ]),
            Row::new(vec![
                Text::styled("Speed", Modifier::BOLD),
                Text::raw(fmt_speed(combatant.speed())),
            ]),
            Row::new(vec![
                Text::styled("Proficiency Bonus", Modifier::BOLD),
                Text::raw(format!("{:+}", combatant.proficiency_bonus())),
            ]),
        ],
        vec![
            Constraint::Percentage(50), // stat name
            Constraint::Percentage(50), // stat value
        ],
    )
        .fg(THEME.foreground)
}

/// A widget similar to [`StatBlock`] that displays relevant combat information.
///
/// [`StatBlock`]: crate::widgets::StatBlock
#[derive(Debug)]
pub struct CombatantBlock<'a> {
    /// The combatant to display.
    combatant: &'a Combatant,
}

impl<'a> CombatantBlock<'a> {
    /// Create a new [`CombatantBlock`] widget.
    pub fn new(combatant: &'a Combatant) -> Self {
        Self { combatant }
    }
}

impl Widget for CombatantBlock<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw bordered box
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(THEME.foreground)
            .title("Combatant Block")
            .render(area, buf);

        let [
            name,
            basic_stats,
            conditions,
            ability_scores,
        ] = Layout::vertical([
            Constraint::Length(1), // name
            Constraint::Min(5),    // basic stats
            Constraint::Fill(1),   // conditions
            Constraint::Min(6),    // ability scores
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .areas(area);

        basic_status_text(self.combatant).render(name, buf);
        Widget::render(basic_stats_table(self.combatant), basic_stats, buf);
        FullConditions::new(self.combatant).render(conditions, buf);
        AbilityScores::new(self.combatant).render(ability_scores, buf);
    }
}
