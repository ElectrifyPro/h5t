use crate::ui_tracker::LabelModeState;
use h5t_core::{Action, Combatant, Tracker};
use ratatui::{prelude::*, widgets::*};

/// Creates a [`Line`] widget for displaying a list of actions.
fn action_line(actions: Action) -> Line<'static> {
    /// Format multiple actions in a compact way (e.g. `Ax4,R`).
    fn fmt_action(label: &str, count: u32) -> String {
        if count <= 3 {
            label.repeat(count as usize)
        } else {
            format!("{}x{}", label, count)
        }
    }

    let mut spans = Vec::new();
    if actions.actions > 0 {
        spans.push(Span::styled(fmt_action("A", actions.actions), Color::Green));
        spans.push(Span::raw(","));
    }
    if actions.bonus_actions > 0 {
        spans.push(Span::styled(fmt_action("BA", actions.bonus_actions), Color::Rgb(255, 165, 0)));
        spans.push(Span::raw(","));
    }
    if actions.reactions > 0 {
        spans.push(Span::styled(fmt_action("R", actions.reactions), Color::Magenta));
        spans.push(Span::raw(","));
    }
    spans.pop(); // remove the last comma
    Line::from(spans)
}

/// Creates a [`Table`] widget for displaying the combatants in the tracker.
fn combatant_table<'a>(widget: &'a TrackerWidget) -> Table<'a> {
    /// Builds a table [`Row`] for a combatant.
    fn combatant_row(label: Option<char>, combatant: &Combatant) -> Row {
        let label_text = label
            .map(|l| Text::from(format!("{}", l)).bold())
            .unwrap_or_default();
        let hp_color = Color::Rgb(
            (255.0 - combatant.hit_points as f32 / combatant.max_hit_points() as f32 * 255.0) as u8,
            (combatant.hit_points as f32 / combatant.max_hit_points() as f32 * 255.0) as u8,
            0,
        );
        Row::new([
            label_text,
            Text::from(combatant.name()),
            action_line(combatant.actions).into(),
            Line::from(vec![
                Span::styled(format!("{}", combatant.hit_points), hp_color),
                Span::raw(format!(" / {}", combatant.max_hit_points())),
            ]).into(),
        ])
    }

    Table::new(
        widget.tracker.combatants.iter()
            .enumerate()
            .map(|(i, combatant)| {
                let is_current_turn = i == widget.tracker.turn;
                let label = widget.label_state.labels.get_by_right(&i).copied();
                let is_label_selected = widget.label_state.selected.contains(&label.unwrap_or_default());

                let row = combatant_row(label, combatant);
                let style = match (is_current_turn, is_label_selected) {
                    (true, true) => Style::default().bold().bg(Color::Gray),
                    (true, false) => Style::default().bg(Color::Rgb(0, 48, 130)),
                    (false, true) => Style::default().bold().bg(Color::Rgb(128, 85, 0)),
                    (false, false) => Color::Reset.into(),
                };
                row.style(style)
            }),
        [
            Constraint::Length(2), // label mode
            Constraint::Fill(2),   // name
            Constraint::Fill(1),   // actions
            Constraint::Fill(1),   // hp / max hp
        ],
    )
        .header(Row::new([
            Text::raw(""),
            Text::from("Name").centered(),
            Text::from("Actions").centered(),
            Text::from("HP / Max HP").centered(),
        ]).bold())
}

/// A widget to render the initiative tracker's state.
#[derive(Debug)]
pub struct TrackerWidget<'a> {
    /// The tracker to display.
    pub tracker: &'a Tracker,

    /// State for label mode.
    pub label_state: LabelModeState,
}

impl<'a> TrackerWidget<'a> {
    /// Create a new [`TrackerWidget`] widget.
    pub fn new(tracker: &'a Tracker) -> Self {
        Self { tracker, label_state: LabelModeState::default() }
    }

    /// Create a new [`TrackerWidget`] widget with the given labels.
    pub fn with_labels(tracker: &'a Tracker, label: LabelModeState) -> Self {
        Self { tracker, label_state: label }
    }
}

/// Returns the maximum number of combatants that can be displayed in the tracker widget, given the
/// size of the widget.
pub(crate) fn max_combatants(size: Size) -> usize {
    size.height as usize - 6 // 2 for upper and lower borders, 4 for header, spacing, etc.
}

impl<'a> Widget for TrackerWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw bordered box for the tracker
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White))
            .title("Initiative Tracker")
            .render(area, buf);

        let layout = Layout::vertical([
            Constraint::Length(2), // round and turn
            Constraint::Fill(1),
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .split(area);
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

        Widget::render(combatant_table(&self), combatants, buf);
    }
}
