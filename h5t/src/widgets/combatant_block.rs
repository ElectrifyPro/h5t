use crate::{
    theme::{Rgb, THEME},
    widgets::{conditions::FullConditions, AbilityScores, HitPoints, fmt_speed},
};
use h5t_core::Combatant;
use ratatui::{prelude::*, widgets::*};

/// Creates a [`Text`] widget for displaying the combatant's name, group, and whether they are dead.
fn identifiers(group_color: Option<Rgb>, combatant: &Combatant) -> Text<'_> {
    let group_color = group_color.unwrap_or(THEME.foreground);
    Text::from(vec![
        Line::from(if combatant.hit_points <= 0 {
            vec![
                Span::styled(combatant.name(), group_color),
                Span::raw(" "),
                Span::styled("(Dead)", THEME.dead),
            ]
        } else {
            vec![
                Span::styled(combatant.name(), group_color),
            ]
        }),
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

        identifiers(self.group_color, self.combatant).render(name, buf);
        Widget::render(basic_stats_table(self.combatant), basic_stats, buf);
        FullConditions::new(self.combatant).render(conditions, buf);
        AbilityScores::new(&self.combatant.kind).render(ability_scores, buf);
    }
}
