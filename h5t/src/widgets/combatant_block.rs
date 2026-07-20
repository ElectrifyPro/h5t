use crate::{
    theme::{Rgb, THEME},
    widgets::{conditions::FullConditions, AbilityScores, HitPoints, fmt_speed},
};
use h5t_core::{Combatant, Health};
use ratatui::{prelude::*, widgets::*};

/// Creates a [`Line`] widget for displaying the combatant's name and their health status.
fn name_and_health(group_color: Rgb, combatant: &Combatant) -> Line<'_> {
    let status_span = match combatant.health {
        Health::Hp(_) => return Line::styled(combatant.name(), group_color),
        Health::Downed { .. } => Span::styled("(Downed)", THEME.warning),
        Health::Stabilized => Span::styled("(Stabilized)", THEME.success),
        Health::Dead => Span::styled("(Dead)", THEME.dead)
    };

    Line::from_iter([
        Span::styled(combatant.name(), group_color),
        Span::raw(" "),
        status_span,
    ])
}

/// Creates a [`Text`] widget for displaying the combatant's name, group, and whether they are dead.
fn identifiers(group_color: Rgb, combatant: &Combatant) -> Text<'_> {
    Text::from(vec![
        name_and_health(group_color, combatant),
        Line::from(vec![
            Span::raw("Group: "),
            if let Some(group) = &combatant.group {
                Span::styled(group, group_color)
            } else {
                Span::styled("(none)", Modifier::ITALIC)
            },
        ]),
    ])
        .bold()
}

/// Creates a [`Table`] widget for displaying a monster's basic statistics.
fn basic_stats_table(combatant: &Combatant) -> Table<'_> {
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
    /// The combatant's group color, if any.
    group_color: Option<Rgb>,

    /// The combatant to display.
    combatant: &'a Combatant,
}

impl<'a> CombatantBlock<'a> {
    /// Create a new [`CombatantBlock`] widget.
    pub fn new(group_color: Option<Rgb>, combatant: &'a Combatant) -> Self {
        Self { group_color, combatant }
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
            Constraint::Length(2), // identifiers
            Constraint::Length(4), // basic stats
            Constraint::Fill(1),   // conditions
            Constraint::Length(7), // ability scores
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .areas(area);

        let group_color = self.group_color.unwrap_or(THEME.foreground);
        identifiers(group_color, self.combatant).render(name, buf);
        Widget::render(basic_stats_table(self.combatant), basic_stats, buf);
        FullConditions::new(self.combatant).render(conditions, buf);
        AbilityScores::new(&self.combatant.kind).render(ability_scores, buf);
    }
}
