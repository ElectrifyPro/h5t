use canvas::Canvas;
use crate::theme::{Rgb, THEME};
use ratatui::{layout::Flex, prelude::*, widgets::*};
use super::popup_area;

/// A popup to get a line of input from the user.
///
/// This widget doesn't actually handle input, it simply acts as a container for the input field.
///
/// # Example
///
/// ```text
///       prompt
///         |
///   vvvvvvvvvvvvv
///  ╭Damage amount╮
///  │ 350█  HP    │
///  ╰─────────────╯
///    ^^^^  ^^
///     | |  |
/// value |  suffix
///       |
///  fake cursor
/// ```
pub struct Input<'a> {
    /// The color of the border.
    color: Rgb,

    /// The prompt to display as the title of the input box.
    prompt: &'a str,

    /// The text that the user has entered.
    value: &'a str,

    /// The suffix to display after the input value, indicating the expected format / unit of the
    /// input.
    suffix: Option<&'a str>,

    /// Maximum length of the input field.
    max_length: usize,

    /// Whether to render the widget in an active state.
    active: bool,
}

impl<'a> Input<'a> {
    /// Create a new [`Input`] popup with all the required fields.
    pub fn new(
        color: Rgb,
        prompt: &'a str,
        value: &'a str,
        max_length: usize,
        active: bool,
    ) -> Self {
        Self {
            color,
            prompt,
            value,
            suffix: None,
            max_length,
            active,
        }
    }

    /// Set the suffix to display after the input value, indicating the expected format / unit of
    /// the input.
    pub fn try_set_suffix(mut self, suffix: Option<&'a str>) -> Self {
        self.suffix = suffix;
        self
    }
}

impl Widget for Input<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let suffix = self.suffix.unwrap_or("");

        // center the input
        let size = (
            if suffix.is_empty() {
                self.prompt.len()
                    .max(self.max_length + 2) as u16 + 2
            } else {
                // box expands to fit the prompt, or the max text input length + suffix + 2 (padding
                // between them) + 2 (margin around them)
                //
                // outside +2 includes the horizontal borders
                self.prompt.len()
                    .max(suffix.len() + self.max_length + 2 + 2) as u16 + 2
            },
            3, // 2 for borders, 1 for text
        );
        let area = popup_area(area, Flex::Center, Flex::End, size, 0);

        // clear the area
        Clear.render(area, buf);
        Widget::render(
            Canvas::default()
                .background_color(THEME.background.into())
                .paint(|_| ()),
            area,
            buf,
        );

        let (color, theme) = if self.active {
            (self.color, THEME)
        } else {
            (self.color.mix(THEME.background), THEME.dim())
        };

        // draw bordered box for the input field
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(color)
            .title(self.prompt)
            .render(area, buf);

        let text_area = area.inner(Margin::new(2, 1));

        // show input value underlined
        Span::raw(format!("{}{}", self.value, " ".repeat(self.max_length.saturating_sub(self.value.len()))))
            .style(theme.foreground)
            .patch_style(Modifier::UNDERLINED)
            .render(text_area, buf);

        // display fake cursor
        let cursor_x = text_area.x + self.value.len() as u16;
        let cursor_y = text_area.y;

        buf.cell_mut((cursor_x, cursor_y))
            .expect("cursor out of bounds")
            .set_bg(theme.foreground.into());

        let [_, suffix_area] = Layout::horizontal([
            Constraint::Length(self.max_length as u16),
            Constraint::Length(suffix.len() as u16),
        ])
            .spacing(2)
            .areas(text_area);

        // show suffix
        Text::raw(suffix)
            .style(theme.foreground)
            .render(suffix_area, buf);
    }
}
