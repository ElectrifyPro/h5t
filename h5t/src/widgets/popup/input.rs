use crate::{theme::{Rgb, THEME}, widgets::popup::Popup};
use ratatui::prelude::*;

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
        let content_width = if suffix.is_empty() {
            self.max_length as u16
        } else {
            // +2 padding between input and suffix
            (2 + self.max_length + suffix.len()) as u16
        };
        let popup = Popup::new(self.color, self.prompt, content_width, 1, self.active);
        let block_area = popup.block_area(area);
        popup.render(area, buf);

        let theme = if self.active {
            THEME
        } else {
            THEME.dim()
        };

        let text_area = block_area.inner(Margin::new(2, 1));

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
