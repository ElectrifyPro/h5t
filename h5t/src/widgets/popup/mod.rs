//! Widgets that pop up and cover the screen.

pub mod input;
pub mod multiselect;
pub mod select;

use canvas::Canvas;
use crate::theme::{Rgb, THEME};
pub use input::Input;
pub use multiselect::Multiselect;
pub use select::Select;
use ratatui::{layout::Flex, prelude::*, widgets::*};

/// Computes the area to render a popup in, given horizontal and vertical alignment requirements
/// and the popup size.
pub(crate) fn popup_area(
    area: Rect,
    horizontal: Flex,
    vertical: Flex,
    size: (u16, u16),
    margin: u16,
) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(size.0)])
        .flex(horizontal)
        .margin(margin)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(size.1)])
        .flex(vertical)
        .margin(margin)
        .areas(area);

    area
}

/// A generic popup that shrinks to fit its content.
pub struct Popup<'a> {
    /// The color of the border.
    color: Rgb,

    /// The prompt to display as the title of the popup.
    prompt: &'a str,

    /// Whether to render the widget in an active state.
    active: bool,

    /// The size of the popup block.
    block_size: (u16, u16),
}

impl<'a> Popup<'a> {
    /// Create a new [`Popup`] with all the required fields.
    pub fn new(
        color: Rgb,
        prompt: &'a str,
        // the minimum width of the content box (not including the prompt or block borders)
        content_width: u16,
        // the minimum height of the content box (not including the prompt or block borders)
        content_height: u16,
        active: bool,
    ) -> Self {
        // center widget in `area`
        // left border (1) + left padding + (1) + right border (1) + right padding (1) = 4
        let block_width = 4 + content_width;
        let block_size = (
            block_width.max(prompt.len() as u16 + 2), // left border + right border = 2
            // top (1) + bottom border (1) = 2
            2 + content_height,
        );

        Self { color, prompt, active, block_size }
    }

    /// Returns the area containing the entire popup block.
    pub fn block_area(&self, viewport: Rect) -> Rect {
        popup_area(viewport, Flex::Center, Flex::Center, self.block_size, 0)
    }
}

impl Widget for Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = self.block_area(area);

        // clear the area
        Clear.render(area, buf);
        Widget::render(
            Canvas::default()
                .background_color(THEME.background.into())
                .paint(|_| ()),
            area,
            buf,
        );

        let color = if self.active {
            self.color
        } else {
            self.color.mix(THEME.background)
        };

        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(color)
            .title(self.prompt)
            .render(area, buf);
    }
}
