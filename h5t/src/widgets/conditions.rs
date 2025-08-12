use h5t_core::{Combatant, Condition, ConditionKind};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

/// Returns a unique color for the condition.
fn condition_color(kind: ConditionKind) -> Color {
    match kind {
        ConditionKind::Blinded => Color::White,
        ConditionKind::Charmed => Color::Magenta,
        ConditionKind::Deafened => Color::Rgb(255, 165, 0), // orange
        ConditionKind::Exhaustion => Color::Rgb(0, 110, 0), // dark green
        ConditionKind::Frightened => Color::Yellow,
        ConditionKind::Grappled => Color::Rgb(255, 80, 0), // dark orange
        ConditionKind::Incapacitated => Color::LightBlue,
        ConditionKind::Invisible => Color::Rgb(200, 200, 200), // light gray
        ConditionKind::Paralyzed => Color::Rgb(0, 0, 85), // dark blue
        ConditionKind::Petrified => Color::DarkGray,
        ConditionKind::Poisoned => Color::Rgb(0, 110, 0), // dark green
        ConditionKind::Prone => Color::Rgb(255, 165, 140), // light orange
        ConditionKind::Restrained => Color::Rgb(80, 165, 0), // dark brown
        ConditionKind::Stunned => Color::LightBlue,
        ConditionKind::Unconscious => Color::Rgb(0, 0, 80), // dark blue
    }
}

/// A widget to display a combatant's active conditions in a compact form.
#[derive(Debug)]
pub struct CompactConditions<'a> {
    /// The conditions to display.
    pub current: &'a [Condition],
}

impl<'a> CompactConditions<'a> {
    /// Create a new [`Conditions`] widget from a [`Combatant`].
    pub fn new(combatant: &'a Combatant) -> Self {
        Self {
            current: &combatant.conditions,
        }
    }

    /// Creates a [`Line`] widget containing the hit points display.
    pub fn line(&self) -> Line<'static> {
        /// Create a [`Span`] for each condition.
        fn make_span(condition: &Condition) -> Span<'static> {
            Span::styled(
                if let Some(rounds_left) = condition.duration.rounds_left() {
                    format!("{}:{}", condition.kind.abbreviation(), rounds_left)
                } else {
                    // infinite duration
                    format!("{}", condition.kind.abbreviation())
                },
                condition_color(condition.kind),
            )
        }

        let conditions = self
            .current
            .iter()
            .map(make_span)
            .intersperse(Span::raw(","))
            .collect::<Vec<_>>();
        Line::from(conditions)
    }
}

impl Widget for CompactConditions<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.line().render(area, buf);
    }
}

/// A widget to display all a combatant's active conditions in a table.
#[derive(Debug)]
pub struct FullConditions<'a> {
    /// The conditions to display.
    pub current: &'a [Condition],
}

impl<'a> FullConditions<'a> {
    /// Create a new [`FullConditions`] widget from a [`Combatant`].
    pub fn new(combatant: &'a Combatant) -> Self {
        Self {
            current: &combatant.conditions,
        }
    }

    /// Creates a [`Table`] widget containing the conditions.
    pub fn table(&self) -> Table<'static> {
        let rows = self
            .current
            .iter()
            .map(|condition| {
                Row::new(vec![
                    Text::styled(condition.kind.to_string(), condition_color(condition.kind)),
                    Text::raw(if let Some(rounds_left) = condition.duration.rounds_left() {
                        format!("{}", rounds_left)
                    } else {
                        "∞".to_string()
                    }),
                ])
            })
            .collect::<Vec<_>>();

        Table::new(rows, [Constraint::Length(13), Constraint::Length(11)])
            .header(Row::new([Text::styled("Condition", Modifier::BOLD), Text::styled("Rounds left", Modifier::BOLD)]))
    }
}

impl Widget for FullConditions<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(self.table(), area, buf);
    }
}
