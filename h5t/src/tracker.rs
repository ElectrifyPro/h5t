use h5t_core::{CombatantKind, Tracker};
use ratatui::{prelude::*, widgets::*};

use crate::monster::MonsterCard;

/// Creates a [`Table`] widget for displaying the combatants in the tracker.
fn combatant_table(tracker: &Tracker) -> Table {
    Table::new(
        tracker.combatants.iter()
            .enumerate()
            .map(|(i, combatant)| {
                let hp_color = Color::Rgb(
                    (255.0 - combatant.hit_points as f32 / combatant.max_hit_points() as f32 * 255.0) as u8,
                    (combatant.hit_points as f32 / combatant.max_hit_points() as f32 * 255.0) as u8,
                    0,
                );
                let row = Row::new([
                    Text::from(combatant.name()),
                    Line::from(vec![
                        Span::styled(format!("{}", combatant.hit_points), hp_color),
                        Span::raw(format!(" / {}", combatant.max_hit_points())),
                    ]).into(),
                ]);
                if i == tracker.turn {
                    row.style(Style::default().bg(Color::Rgb(0, 48, 130)))
                } else {
                    row
                }
            }),
        [
            Constraint::Fill(1),    // name
            Constraint::Length(14), // hp / max hp
        ],
    )
        .header(Row::new([
            Text::from("Name").centered(),
            Text::from("HP / Max HP").centered(),
        ]).bold())
}

/// A widget to render the initiative tracker's state.
#[derive(Debug)]
pub struct TrackerWidget<'a> {
    /// The tracker to display.
    pub tracker: &'a Tracker,
}

impl<'a> TrackerWidget<'a> {
    /// Create a new [`TrackerWidget`] widget.
    pub fn new(tracker: &'a Tracker) -> Self {
        Self { tracker }
    }
}

impl<'a> Widget for TrackerWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
            .split(area);
        let [tracker, stat_block] = [layout[0], layout[1]];

        // draw bordered boxes for the tracker and the monster card
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White))
            .title("Initiative Tracker")
            .render(tracker, buf);

        let CombatantKind::Monster(monster) = &self.tracker.current_combatant().kind;
        MonsterCard::new(monster).render(stat_block, buf);

        let layout = Layout::vertical([
            Constraint::Length(2), // round and turn
            Constraint::Fill(1),
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .split(tracker);
        let [round_and_turn, combatants] = [layout[0], layout[1]];

        let text = vec![
            Line::styled(format!("Round: {}", self.tracker.round + 1), Modifier::BOLD),
            Line::styled(
                format!("Turn: {}/{}", self.tracker.turn + 1, self.tracker.combatants.len()),
                Modifier::BOLD
            ),
        ];
        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .render(round_and_turn, buf);

        Widget::render(combatant_table(self.tracker), combatants, buf);
    }
}
