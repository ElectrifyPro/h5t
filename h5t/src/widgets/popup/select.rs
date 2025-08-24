use canvas::Canvas;
use crate::{selectable::Selectable, theme::THEME, ui::LABELS};
use ratatui::{layout::Flex, prelude::*, widgets::*};
use super::popup_area;

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
    selected: &'a T,

    /// Whether to render the widget in an active state.
    active: bool,
}

impl<'a, T> Select<'a, T> {
    /// Create a new [`Select`] popup with all the required fields.
    pub fn new(prompt: &'a str, selected: &'a T, active: bool) -> Self {
        Self { prompt, selected, active }
    }
}

impl<T: Selectable> Widget for Select<'_, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // center widget
        // 4 for borders and text padding, 2 for space for labels
        let content_width = 4 + 2 + T::variants()
            .map(|v| v.to_string().len())
            .max()
            .unwrap_or(0) as u16;
        let size = (
            content_width.max(self.prompt.len() as u16 + 2),
            // 2 for top and bottom border
            2 + T::N as u16,
        );
        let area = popup_area(area, Flex::Center, Flex::Center, size, 0);

        // clear the area
        Clear.render(area, buf);
        Widget::render(
            Canvas::default()
                .background_color(THEME.background.into())
                .paint(|_| ()),
            area,
            buf,
        );

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

                    if *self.selected == option {
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
            .block(Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(theme.foreground)
                .title(self.prompt)
                .padding(Padding::symmetric(1, 0)));

        Widget::render(widget, area, buf);
    }
}
