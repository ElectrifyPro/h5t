use crate::{theme::{Rgb, THEME}, widgets::popup::{Popup, SizedWidget}};
use ratatui::prelude::*;

/// The text content inside the input popup.
struct InputInner<'a> {
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

impl<'a> InputInner<'a> {
    /// Create a new [`InputInner`] with all the required fields.
    pub fn new(
        value: &'a str,
        max_length: usize,
        active: bool,
    ) -> Self {
        Self {
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

impl SizedWidget for InputInner<'_> {
    fn width(&self) -> u16 {
        if let Some(suffix) = self.suffix {
            // 2 for the spacing between the input and the suffix
            (2 + self.max_length + suffix.len()) as u16
        } else {
            self.max_length as u16
        }
    }

    fn height(&self) -> u16 {
        1
    }
}

impl Widget for InputInner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = if self.active {
            THEME
        } else {
            THEME.dim()
        };

        // show input value underlined
        Span::raw(format!("{}{}", self.value, " ".repeat(self.max_length.saturating_sub(self.value.len()))))
            .style(theme.foreground)
            .patch_style(Modifier::UNDERLINED)
            .render(area, buf);

        // display fake cursor
        let cursor_x = area.x + self.value.len() as u16;
        let cursor_y = area.y;

        buf.cell_mut((cursor_x, cursor_y))
            .expect("cursor out of bounds")
            .set_bg(theme.foreground.into());

        let suffix = self.suffix.unwrap_or("");
        let [_, suffix_area] = Layout::horizontal([
            Constraint::Length(self.max_length as u16),
            Constraint::Length(suffix.len() as u16),
        ])
            .spacing(2)
            .areas(area);

        // show suffix
        Text::raw(suffix)
            .style(theme.foreground)
            .render(suffix_area, buf);
    }
}

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
        let widget = InputInner::new(self.value, self.max_length, self.active)
            .try_set_suffix(self.suffix);
        let popup = Popup::new(self.color, Some(self.prompt), self.active, widget);
        popup.render(area, buf);
    }
}
