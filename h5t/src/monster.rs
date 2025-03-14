use h5t_core::{monster::{AbilityScores, Size, Speed, Type, Usage}, Monster};
use ratatui::{prelude::*, widgets::*};

/// Formats a modifier-like value in the form `(±X)`.
fn fmt_mod(paren: bool, modifier: i32) -> String {
    if paren {
        format!("({:+})", modifier)
    } else {
        format!("{:+}", modifier)
    }
}

/// Creates a [`Paragraph`] widget for displaying the monster's name and type.
fn name_and_type_paragraph(monster: &Monster) -> Paragraph {
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
}

/// Creates a [`Table`] widget for displaying a monster's basic statistics.
fn basic_stats_table(monster: &Monster) -> Table {
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
                Text::raw(monster.armor_class.value.to_string()),
            ]),
            Row::new(vec![
                Text::styled("Hit Points", Modifier::BOLD),
                Text::raw(format!("{} ({})", monster.hit_points, monster.hit_points_roll)),
            ]),
            Row::new(vec![
                Text::styled("Speed", Modifier::BOLD),
                Text::raw(fmt_speed(&monster.speed)),
            ]),
            Row::new(vec![
                Text::styled("Challenge", Modifier::BOLD),
                Text::raw(fmt_cr(monster.challenge_rating, monster.xp)),
            ]),
            Row::new(vec![
                Text::styled("Proficiency Bonus", Modifier::BOLD),
                Text::raw(fmt_mod(false, monster.proficiency_bonus)),
            ]),
        ],
        vec![
            Constraint::Percentage(50), // stat name
            Constraint::Percentage(50), // stat value
        ],
    )
}

/// Creates a [`Table`] widget for displaying a monster's ability scores.
fn ability_scores_table(monster: &Monster) -> Table {
    let (str, dex, con, int, wis, cha) = (
        monster.scores.strength,
        monster.scores.dexterity,
        monster.scores.constitution,
        monster.scores.intelligence,
        monster.scores.wisdom,
        monster.scores.charisma,
    );

    /// Helper to build a row for the ability scores table.
    fn row(odd: bool, ability: &str, score: i32) -> Row {
        // more green for high scores, more red for low scores
        // 0: (255, 0, 0)
        // 10: (255, 255, 255)
        // 20: (0, 255, 0)
        let color = Color::Rgb(
            (510.0 - 255.0 / 10.0 * score as f32).min(255.0) as u8,
            (255.0 / 10.0 * score as f32).min(255.0) as u8,
            (255.0 - (255.0 / 10.0 * score as f32 - 255.0).abs()).max(0.0) as u8,
        );

        Row::new(vec![
            Text::styled(ability, Modifier::BOLD),
            Text::styled(score.to_string(), color),
            Text::styled(fmt_mod(true, AbilityScores::modifier(score)), color),
        ])
            .style(Style::default().bg(if odd { Color::DarkGray } else { Color::Black }))
    }

    Table::new(
        vec![
            row(false, "STR", str),
            row(true, "DEX", dex),
            row(false, "CON", con),
            row(true, "INT", int),
            row(false, "WIS", wis),
            row(true, "CHA", cha),
        ],
        vec![
            Constraint::Percentage(50), // ability abbreviation
            Constraint::Length(3),         // ability score
            Constraint::Min(4),         // modifier
        ],
    )
}

/// Creates a [`Paragraph`] widget for displaying a monster's special abilities.
fn special_abilities_paragraph(monster: &Monster) -> Paragraph {
    use itertools::Itertools;

    let text = monster
        .special_abilities
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
        .wrap(Wrap { trim: true })
}

/// A card widget for displaying a monster's statistics.
#[derive(Debug)]
pub struct MonsterCard<'a> {
    /// The monster to display.
    pub monster: &'a Monster,
}

impl<'a> MonsterCard<'a> {
    /// Create a new [`MonsterCard`] widget.
    pub fn new(monster: &'a Monster) -> Self {
        Self { monster }
    }
}

impl<'a> Widget for MonsterCard<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw bordered box
        Widget::render(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::White))
                .title("Monster"),
            area,
            buf,
        );

        let layout = Layout::vertical([
            Constraint::Length(2), // name and type
            Constraint::Length(4), // basic stats
            Constraint::Length(6), // ability scores
            Constraint::Min(1), // special abilities
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .split(area);
        let [
            name,
            basic_stats,
            ability_scores,
            special_abilities
        ] = [layout[0], layout[1], layout[2], layout[3]];

        Widget::render(name_and_type_paragraph(self.monster), name, buf);
        Widget::render(basic_stats_table(self.monster), basic_stats, buf);
        Widget::render(ability_scores_table(self.monster), ability_scores, buf);
        Widget::render(special_abilities_paragraph(self.monster), special_abilities, buf);
    }
}
