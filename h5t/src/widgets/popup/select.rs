use crate::{selectable::Selectable, theme::THEME, view::LABELS, widgets::popup::Popup};
use ratatui::{prelude::*, widgets::*};

/// A popup that displays a selection prompt for an enum. Like [`Multiselect`], but for a single
/// option.
///
/// This widget doesn't actually handle input, it simply acts as a container for the input.
///
/// [`Multiselect`]: super::Multiselect
pub struct Select<'a, T> {
    /// The prompt to display as the title of the input box.
    prompt: &'a str,

    /// The selected variant.
    selected: Option<&'a T>,

    /// Whether to render the widget in an active state.
    active: bool,
}

impl<'a, T> Select<'a, T> {
    /// Create a new [`Select`] popup with our without a field preselected.
    pub fn new(prompt: &'a str, selected: Option<&'a T>, active: bool) -> Self {
        Self { prompt, selected, active }
    }

    /// Create a new [`Select`] popup with a field preselected.
    pub fn with_selected(prompt: &'a str, selected: &'a T, active: bool) -> Self {
        Self { prompt, selected: Some(selected), active }
    }
}

impl<T: Selectable> Widget for Select<'_, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_width = 2 + T::variants() // +2 for the labels
            .map(|v| v.to_string().len())
            .max()
            .unwrap_or(0) as u16;
        let popup = Popup::new(THEME.foreground, self.prompt, content_width, T::N as u16, self.active);
        let block_area = popup.block_area(area);
        popup.render(area, buf);

        let theme = if self.active {
            THEME
        } else {
            THEME.dim()
        };
        let widget = Table::new(
            LABELS.chars()
                .zip(T::variants())
                .map(|(label, option)| {
                    let mut style = Style::default()
                        .fg(theme.foreground.into());

                    if let Some(selected) = self.selected && *selected == option {
                        style = style.bold().bg(theme.select.into());
                    }

                    Row::new(vec![
                        Text::styled(label.to_string(), Modifier::BOLD),
                        Text::raw(option.to_string()),
                    ]).style(style)
                }),
            [
                Constraint::Length(1),
                Constraint::Fill(1),
            ],
        )
            .block(Block::new().padding(Padding::symmetric(2, 1)));

        Widget::render(widget, block_area, buf);
    }
}
