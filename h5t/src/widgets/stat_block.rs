use crate::{theme::THEME, widgets::{AbilityScores, fmt_speed}};
use h5t_core::{monster::{Size, Type, Usage}, CombatantKind, Monster};
use ratatui::{prelude::*, widgets::*};

/// Creates a [`Paragraph`] widget for displaying the [`Monster`]'s name and type.
fn name_and_type_paragraph(monster: &Monster) -> Paragraph<'_> {
    let size = match monster.size {
        Size::Tiny => "Tiny",
        Size::Small => "Small",
        Size::Medium => "Medium",
        Size::Large => "Large",
        Size::Huge => "Huge",
        Size::Gargantuan => "Gargantuan",
    };
    let r#type = match monster.r#type {
        Type::Aberration => "Aberration",
        Type::Beast => "Beast",
        Type::Celestial => "Celestial",
        Type::Construct => "Construct",
        Type::Dragon => "Dragon",
        Type::Elemental => "Elemental",
        Type::Fey => "Fey",
        Type::Fiend => "Fiend",
        Type::Giant => "Giant",
        Type::Humanoid => "Humanoid",
        Type::Monstrosity => "Monstrosity",
        Type::Ooze => "Ooze",
        Type::Plant => "Plant",
        Type::Undead => "Undead",
        Type::Other => "Other",
    };
    let subtype = if let Some(subtype) = &monster.subtype {
        format!(" ({})", subtype)
    } else {
        "".to_string()
    };

    Paragraph::new(vec![
        Span::styled(&monster.name, Modifier::BOLD).into(),
        Line::from(vec![
            Span::raw(size),
            Span::raw(" "),
            Span::raw(r#type),
            Span::raw(", "),
            Span::raw(&monster.alignment),
            Span::raw(subtype),
        ]).style(Modifier::ITALIC),
    ])
        .fg(THEME.foreground)
}

/// Creates a [`Table`] widget for displaying a creature's basic statistics.
fn basic_stats_table(creature: &CombatantKind) -> Table<'_> {
    /// Formats a challenge rating.
    fn fmt_cr(cr: f32, xp: i32) -> String {
        let cr_value = if cr == 0.0 {
            "0".to_string()
        } else if cr < 1.0 {
            format!("1/{}", 1.0 / cr)
        } else {
            cr.to_string()
        };

        format!("{} ({} XP)", cr_value, xp)
    }

    Table::new(
        vec![
            Row::new(vec![
                Text::styled("Armor Class", Modifier::BOLD),
                Text::raw(creature.armor_class().to_string()),
            ]),
            Row::new(vec![
                Text::styled("Hit Points", Modifier::BOLD),
                match creature {
                    CombatantKind::Character(character) => {
                        Text::raw(format!("{}", character.hit_points))
                    },
                    CombatantKind::Monster(monster) => {
                        Text::raw(format!("{} ({})", monster.hit_points, monster.hit_points_roll))
                    },
                }
            ]),
            Row::new(vec![
                Text::styled("Speed", Modifier::BOLD),
                Text::raw(fmt_speed(&creature.speed())),
            ]),
            match creature {
                CombatantKind::Character(character) => Row::new(vec![
                    Text::styled("Level", Modifier::BOLD),
                    Text::raw(format!("{}", character.level)),
                ]),
                CombatantKind::Monster(monster) => Row::new(vec![
                    Text::styled("Challenge", Modifier::BOLD),
                    Text::raw(fmt_cr(monster.challenge_rating, monster.xp)),
                ]),
            },
            Row::new(vec![
                Text::styled("Proficiency Bonus", Modifier::BOLD),
                Text::raw(format!("{:+}", creature.proficiency_bonus())),
            ]),
        ],
        vec![
            Constraint::Percentage(50), // stat name
            Constraint::Percentage(50), // stat value
        ],
    )
        .fg(THEME.foreground)
}

/// Creates a [`Paragraph`] widget for displaying a [`Monster`]'s traits.
fn traits_paragraph(monster: &Monster) -> Paragraph<'_> {
    use itertools::Itertools;

    let text = monster
        .traits
        .iter()
        .map(|ability| {
            let constraint = match ability.usage {
                Usage::PerDay(count) => format!(" ({}/Day). ", count),
                Usage::RechargeAfterRest => " (Recharges after a Short or Long Rest). ".to_string(),
                Usage::RechargeAfterLongRest => " (Recharges after a Long Rest). ".to_string(),
                Usage::AtWill => ". ".to_string(),
            };
            Line::from(vec![
                Span::styled(&ability.name, Modifier::BOLD | Modifier::ITALIC),
                Span::styled(constraint, Modifier::BOLD | Modifier::ITALIC),
                Span::raw(&ability.desc),
            ])
        })
        .intersperse(Line::raw(""))
        .collect::<Vec<_>>();
    Paragraph::new(text)
        .fg(THEME.foreground)
        .wrap(Wrap { trim: true })
}

/// A widget for displaying a monster's stat block.
#[derive(Debug)]
pub struct StatBlock<'a> {
    /// The creature to display.
    pub creature: &'a CombatantKind,
}

impl<'a> StatBlock<'a> {
    /// Create a new [`StatBlock`] widget.
    pub fn new(creature: &'a CombatantKind) -> Self {
        Self { creature }
    }
}

impl Widget for StatBlock<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw bordered box
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(THEME.foreground)
            .title("Monster Stat Block")
            .render(area, buf);

        let [
            name,
            basic_stats,
            ability_scores,
            traits
        ] = Layout::vertical([
            Constraint::Length(2), // name and type
            Constraint::Length(5), // basic stats
            Constraint::Length(7), // ability scores
            Constraint::Min(1),    // traits
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .areas(area);

        match self.creature {
            CombatantKind::Character(character) => {
                Span::styled(&character.name, Modifier::BOLD).render(name, buf);
            },
            CombatantKind::Monster(monster) => {
                name_and_type_paragraph(monster).render(name, buf);
            },
        }
        Widget::render(basic_stats_table(self.creature), basic_stats, buf);
        AbilityScores::new(self.creature).render(ability_scores, buf);
        if let CombatantKind::Monster(monster) = self.creature {
            traits_paragraph(monster).render(traits, buf);
        }
    }
}
