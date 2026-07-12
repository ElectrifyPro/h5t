use crate::{theme::{Rgb, THEME}, widgets::popup::{Popup, SizedWidget}};
use ratatui::prelude::*;

/// The text content inside the input popup.
struct InputInner<'a> {
    /// The text that the user has entered.
    value: &'a str,

    /// The prefix to display before the input value, e.g. a dice expression.
    prefix: Option<&'a str>,

    /// The suffix to display after the input value, e.g. the expected format / unit of the input.
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
            prefix: None,
            suffix: None,
            max_length,
            active,
        }
    }

    /// Set the prefix to display before the input value, e.g. a dice expression.
    pub fn try_set_prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.prefix = prefix;
        self
    }

    /// Set the suffix to display after the input value, e.g. the expected format / unit of the
    /// input.
    pub fn try_set_suffix(mut self, suffix: Option<&'a str>) -> Self {
        self.suffix = suffix;
        self
    }
}

impl SizedWidget for InputInner<'_> {
    fn width(&self) -> u16 {
        let prefix = self.prefix.unwrap_or_default();
        let suffix = self.suffix.unwrap_or_default();

        let mut inner_width = self.max_length;
        if !prefix.is_empty() {
            inner_width += 2 + prefix.len();
        }
        if !suffix.is_empty() {
            inner_width += 2 + suffix.len();
        }

        inner_width as u16
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

        let prefix = self.prefix.unwrap_or_default();
        let suffix = self.suffix.unwrap_or_default();

        // if the prefix / suffix exists, reserve enough space for it + 2 extra separator cells that
        // separate the prefix / suffix between the input. otherwise, the entire space is empty
        let tip_length = |fix: Option<&str>| if let Some(s) = fix && !s.is_empty() {
            s.len() as u16 + 2
        } else {
            0
        };

        let [prefix_area, input_area, suffix_area] = Layout::horizontal([
            Constraint::Length(tip_length(self.prefix)),
            Constraint::Length(self.max_length as u16),
            Constraint::Length(tip_length(self.suffix)),
        ])
            .areas(area);

        // show input value underlined
        Span::raw(format!("{}{}", self.value, " ".repeat(self.max_length.saturating_sub(self.value.len()))))
            .style(theme.foreground)
            .patch_style(Modifier::UNDERLINED)
            .render(input_area, buf);

        // display fake cursor
        let cursor_x = input_area.x + self.value.len() as u16;
        let cursor_y = input_area.y;

        buf.cell_mut((cursor_x, cursor_y))
            .expect("cursor out of bounds")
            .set_bg(theme.foreground.into());

        Text::raw(prefix)
            .alignment(HorizontalAlignment::Left)
            .style(theme.foreground)
            .render(prefix_area, buf);
        Text::raw(suffix)
            .alignment(HorizontalAlignment::Right)
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
///      prompt
///        |
///   vvvvvvvvvvv
///  ╭Damage roll────────────────╮
///  │ d20 + 3  15█  bludgeoning │
///  ╰───────────────────────────╯
///    ^^^^^^^  ^^^  ^^^^^^^^^^^
///       |     | |       |
///       | value |    suffix
///       |       |
///  prefix  fake cursor
/// ```
///
/// The input area between the prefix and suffix has width `max_length` and is underlined in a
/// terminal, with extra padding added.
pub struct Input<'a> {
    /// The color of the border.
    color: Rgb,

    /// The prompt to display as the title of the input box.
    prompt: &'a str,

    /// The text that the user has entered.
    value: &'a str,

    /// The prefix to display before the input value, e.g. a dice expression.
    prefix: Option<&'a str>,

    /// The suffix to display after the input value, e.g. the expected format / unit of the input.
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
            prefix: None,
            suffix: None,
            max_length,
            active,
        }
    }

    /// Set the prefix to display before the input value, e.g. a dice expression.
    pub fn try_set_prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.prefix = prefix;
        self
    }

    /// Set the suffix to display after the input value, e.g. the expected format / unit of the
    /// input.
    pub fn try_set_suffix(mut self, suffix: Option<&'a str>) -> Self {
        self.suffix = suffix;
        self
    }
}

impl Widget for Input<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let widget = InputInner::new(self.value, self.max_length, self.active)
            .try_set_prefix(self.prefix)
            .try_set_suffix(self.suffix);
        let popup = Popup::new(self.color, Some(self.prompt), self.active, widget);
        popup.render(area, buf);
    }
}

// TODO: i want to add rendering tests for Input
// let area = popup_area(a, HorizontalAlignment::Center, VerticalAlignment::Center, (50, 50)); // expected popup size: (17, 3)
// GetInput::<i32>::new("Init", 5, Charset::Numeric)
//     .prefix("d6 + 8")
//     .draw(frame, area);
// let area = popup_area(b, HorizontalAlignment::Center, VerticalAlignment::Center, (50, 50)); // expected popup size: (21, 3)
// GetInput::<i32>::new("Hitdi", 5, Charset::Numeric)
//     .prefix("d6 + 8")
//     .suffix("HP")
//     .draw(frame, area);
// let area = popup_area(c, HorizontalAlignment::Center, VerticalAlignment::Center, (50, 50)); // expected popup size: (13, 3)
// GetInput::<i32>::new("Da", 5, Charset::Numeric)
//     .suffix("HP")
//     .draw(frame, area)
//
//   ╭Init───────────╮
//   │ d6 + 8        │
//   ╰───────────────╯
// ╭Hitdi─────────────╮
// │ d6 + 8  ____  HP │
// ╰──────────────────╯
//     ╭Da─────────╮
//     │        HP │
//     ╰───────────╯
