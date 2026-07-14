use crate::{
    theme::{Rgb, THEME},
    view::battle::LabelModeState,
    widgets::{CompactConditions, HitPoints},
};
use h5t_core::{
    resource::{
        Action,
        BonusAction,
        MovementUsed,
        Reaction,
        Resource,
        ResourcePool,
        SpeedMultiplier,
    },
    Combatant,
    Tracker as CoreTracker,
};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;

/// Creates a [`Text`] widget for displaying the character's remaining movement.
fn movement_speed(combatant: &Combatant) -> Line<'static> {
    let base_speed = combatant.speed();
    let movement_used = MovementUsed::get(&combatant.resource_pool);

    // used to account for dashing
    let speed_multiplier = SpeedMultiplier::get(&combatant.resource_pool);

    // create a `Span` for each speed value
    let make_span = |prefix: &str, speed: Option<i32>| {
        speed
            .map(|speed| speed_multiplier * speed - movement_used)
            .filter(|speed| *speed > 0)
            .map(|speed| Span::from(format!("{} {} ft.", prefix, speed)))
    };

    [
        make_span("", base_speed.walk),
        make_span("B", base_speed.burrow),
        make_span("C", base_speed.climb),
        make_span("F", base_speed.fly),
        make_span("S", base_speed.swim),
    ]
        .into_iter()
        .flatten()
        .intersperse(Span::raw("|"))
        .collect::<Vec<_>>()
        .into()
}

/// Creates a [`Line`] widget for displaying the character's action count.
fn action_line(pool: &ResourcePool) -> Line<'static> {
    /// Format multiple actions in a compact way.
    ///
    /// Example:
    ///
    /// - 0 => "   "
    /// - 1 => "A  "
    /// - 2 => "AA "
    /// - 3 => "AAA"
    /// - 4 => "Ax4"
    fn fmt_action(label: &str, count: i32) -> String {
        match count {
            c if c <= 0 => "   ".to_string(),
            1..=3 => format!("{:<3}", label.repeat(count as usize)),
            _ => format!("{}x{}", label, count),
        }
    }

    let (
        action_count,
        bonus_action_count,
        reaction_count,
    ) = (
        Action::get(pool),
        BonusAction::get(pool),
        Reaction::get(pool),
    );

    Line::from(vec![
        Span::styled(fmt_action("A", action_count), THEME.action),
        Span::styled("|", THEME.foreground),
        Span::styled(fmt_action("B", bonus_action_count), THEME.bonus_action),
        Span::styled("|", THEME.foreground),
        Span::styled(fmt_action("R", reaction_count), THEME.reaction),
    ])
}

/// Creates a [`Table`] widget for displaying the combatants in the tracker.
fn combatant_table<'a>(widget: &'a Tracker) -> Table<'a> {
    /// Builds a table [`Row`] for a combatant.
    fn combatant_row<'a>(
        label: Option<char>,
        group_colors: &HashMap<String, Rgb>,
        combatant: &'a Combatant,
    ) -> Row<'a> {
        let label_text = label
            .map(|l| Text::from(format!("{}", l)).bold())
            .unwrap_or_default();
        let combatant_group_color = combatant.group
            .as_ref()
            .and_then(|group| group_colors.get(group))
            .copied()
            .unwrap_or(THEME.foreground);

        Row::new([
            label_text,
            Text::styled(combatant.name(), combatant_group_color),
            movement_speed(combatant).into(),
            action_line(&combatant.resource_pool).into(),
            HitPoints::new(combatant).line().into(),
            CompactConditions::new(combatant).line().into(),
        ])
    }

    Table::new(
        widget.tracker.combatants.iter()
            .enumerate()
            .skip(widget.scroll_index)
            .map(|(i, combatant)| {
                let is_current_turn = i == widget.tracker.turn;
                let (label, is_label_selected) = if let Some(label_state) = widget.label_state {
                    let label = label_state.labels.get_by_right(&i).copied();
                    (
                        label,
                        label_state.selected_combatants.contains(&i),
                    )
                } else {
                    (None, false)
                };

                let row = combatant_row(label, &widget.groups, combatant);
                let mut style = Style::default().fg(THEME.foreground.into());
                if is_label_selected {
                    style = style.bold();
                }

                let mut bg_color = None;
                if combatant.hit_points <= 0 {
                    bg_color = bg_color
                        .map(|current| THEME.dead.mix(current))
                        .or(Some(THEME.dead));
                }
                if is_current_turn {
                    bg_color = bg_color
                        .map(|current| THEME.primary.mix(current))
                        .or(Some(THEME.primary));
                }
                if is_label_selected {
                    bg_color = bg_color
                        .map(|current| THEME.select.mix(current))
                        .or(Some(THEME.select));
                }

                let bg_color = bg_color.unwrap_or(THEME.background);
                style = style.bg(bg_color.into());

                row.style(style)
            }),
        [
            Constraint::Length(2), // label mode
            Constraint::Fill(2),   // name
            Constraint::Fill(1),   // movement
            Constraint::Fill(1),   // actions
            Constraint::Fill(1),   // hp / max hp
            Constraint::Fill(1),   // conditions
        ],
    )
        .header(
            Row::new([
                Text::raw(""),
                Text::from("Name").centered(),
                Text::from("Movement").centered(),
                Text::from("Actions").centered(),
                Text::from("HP / Max HP").centered(),
                Text::from("Conditions").centered(),
            ])
                .style(THEME.foreground)
                .bold()
        )
}

/// A widget to render the initiative tracker's state.
#[derive(Debug)]
pub struct Tracker<'a> {
    /// The tracker to display.
    pub tracker: &'a CoreTracker,

    /// Groups that combatants can be assigned to.
    pub groups: &'a HashMap<String, Rgb>,

    /// Index of the first combatant listed in the tracker, used to scroll through the initiative
    /// tracker.
    pub scroll_index: usize,

    /// State for label mode.
    pub label_state: Option<&'a LabelModeState>,
}

impl<'a> Tracker<'a> {
    /// Create a new [`Tracker`] widget.
    pub fn new(
        tracker: &'a CoreTracker,
        groups: &'a HashMap<String, Rgb>,
        scroll_index: usize,
    ) -> Self {
        Self {
            tracker,
            groups,
            scroll_index,
            label_state: None,
        }
    }

    /// Create a new [`Tracker`] widget with the given labels.
    pub fn with_labels(
        tracker: &'a CoreTracker,
        groups: &'a HashMap<String, Rgb>,
        scroll_index: usize,
        label_state: &'a LabelModeState,
    ) -> Self {
        Self {
            tracker,
            groups,
            scroll_index,
            label_state: Some(label_state),
        }
    }
}

/// Returns the maximum number of combatants that can be displayed in the tracker widget, given the
/// size of the widget.
pub(crate) fn max_combatants(size: Size) -> usize {
    size.height as usize - 6 // 2 for upper and lower borders, 4 for header, spacing, etc.
}

impl Widget for Tracker<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw bordered box for the tracker
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(THEME.foreground)
            .title("Initiative Tracker")
            .render(area, buf);

        let [round_and_turn, combatants] = Layout::vertical([
            Constraint::Length(2), // round and turn
            Constraint::Fill(1),
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .areas(area);

        let text = vec![
            Line::styled(format!("Round: {}", self.tracker.round + 1), Modifier::BOLD),
            Line::styled(
                format!("Turn: {}/{}", self.tracker.turn + 1, self.tracker.combatants.len()),
                Modifier::BOLD
            ),
        ];
        Paragraph::new(text)
            .style(THEME.foreground)
            .wrap(Wrap { trim: true })
            .render(round_and_turn, buf);

        Widget::render(combatant_table(&self), combatants, buf);
    }
}
