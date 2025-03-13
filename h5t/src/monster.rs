use h5t_core::{monster::{AbilityScores, Usage}, Monster};
use ratatui::{prelude::*, widgets::*};

/// Creates a [`Table`] widget for displaying a monster's ability scores.
fn ability_scores_table(monster: &Monster) -> Table {
    /// Formats an ability modifier in the form `(±X)`.
    fn fmt_mod(modifier: i32) -> String {
        format!("({}{})", if modifier >= 0 { "+" } else { "" }, modifier)
    }

    let (str, dex, con, int, wis, cha) = (
        monster.scores.strength,
        monster.scores.dexterity,
        monster.scores.constitution,
        monster.scores.intelligence,
        monster.scores.wisdom,
        monster.scores.charisma,
    );

    /// Helper to build a row for the ability scores table.
    fn row(ability: &str, score: i32) -> Row {
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
            Text::styled(fmt_mod(AbilityScores::modifier(score)), color),
        ])
    }

    Table::new(
        vec![
            row("STR", str),
            row("DEX", dex),
            row("CON", con),
            row("INT", int),
            row("WIS", wis),
            row("CHA", cha),
        ],
        vec![
            Constraint::Percentage(50), // ability abbreviation
            Constraint::Max(4),         // ability score
            Constraint::Min(4),         // modifier
        ],
    )
}

/// Creates a [`Paragraph`] widget for displaying a monster's special abilities.
fn special_abilities_paragraph(monster: &Monster) -> Paragraph {
    use itertools::Itertools;

    let mut text = monster
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
            Constraint::Length(1), // name
            Constraint::Length(6), // ability scores
            Constraint::Min(1), // special abilities
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .split(area);
        let [name, ability_scores, special_abilities] = [layout[0], layout[1], layout[2]];

        Widget::render(Text::styled(&self.monster.name, Modifier::BOLD), name, buf);
        Widget::render(ability_scores_table(self.monster), ability_scores, buf);
        Widget::render(special_abilities_paragraph(self.monster), special_abilities, buf);
    }
}
