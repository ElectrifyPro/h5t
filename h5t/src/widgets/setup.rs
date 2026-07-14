use crate::{
    theme::{Rgb, THEME},
    view::setup::SetupInner,
    widgets::{ability_scores::score_to_color, HitPoints},
};
use h5t_core::Combatant;
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;

/// Creates a [`Table`] widget for displaying the combatants in the tracker.
fn combatant_table(widget: Setup) -> Table {
    /// Builds a table [`Row`] for a combatant.
    fn combatant_row<'a>(group_colors: &HashMap<String, Rgb>, combatant: &'a Combatant) -> Row<'a> {
        Row::new([
            Text::from(combatant.name()),
            Text::styled(
                combatant.group.as_deref().unwrap_or_default(),
                combatant.group
                    .as_ref()
                    .and_then(|group| group_colors.get(group))
                    .copied()
                    .unwrap_or(THEME.foreground),
            ),
            Text::styled(
                format!("{:+}", combatant.scores().modifiers().dexterity),
                score_to_color(combatant.scores().dexterity),
            ),
            Text::from(
                combatant.initiative
                    .map(|init| init.to_string())
                    .unwrap_or_default(),
            ),
            HitPoints::new(combatant).line().into(),
        ])
    }

    Table::new(
        widget.inner.combatants.iter()
            .skip(widget.scroll_index)
            .map(|combatant| {
                let is_selected = false;

                let row = combatant_row(&widget.inner.groups, combatant);
                let mut style = Style::default().fg(THEME.foreground.into());

                let mut bg_color = None;
                if is_selected {
                    bg_color = bg_color
                        .map(|current| THEME.primary.mix(current))
                        .or(Some(THEME.primary));
                }

                let bg_color = bg_color.unwrap_or(THEME.background);
                style = style.bg(bg_color.into());

                row.style(style)
            }),
        [
            Constraint::Fill(2), // name
            Constraint::Fill(1), // group
            Constraint::Fill(1), // dexterity modifier
            Constraint::Fill(1), // initiative roll
            Constraint::Fill(1), // hp / max hp
        ],
    )
        .header(
            Row::new([
                Text::from("Name").centered(),
                Text::from("Group").centered(),
                Text::from("DEX Mod").centered(),
                Text::from("Initiative").centered(),
                Text::from("HP / Max HP").centered(),
            ])
                .style(THEME.foreground)
                .bold()
        )
}

/// The widget used to used to setup and add combatants, and roll initiative order.
#[derive(Debug)]
pub struct Setup<'a> {
    /// The setup data for the battle.
    pub inner: &'a SetupInner,

    /// Index of the first combatant listed in the tracker, used to scroll through the initiative
    /// tracker.
    pub scroll_index: usize,
}

impl<'a> Setup<'a> {
    /// Create a new [`Setup`] widget.
    pub fn new(inner: &'a SetupInner, scroll_index: usize) -> Self {
        Self { inner, scroll_index }
    }
}

impl Widget for Setup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(THEME.foreground)
            .title("Setup Battle")
            .render(area, buf);

        let [stat_line, combatants] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .areas(area);

        Line::from(format!("# combatants: {}", self.inner.combatants.len()))
            .style(Style::default().fg(THEME.foreground.into()).bold())
            .render(stat_line, buf);

        Widget::render(combatant_table(self), combatants, buf);
    }
}
