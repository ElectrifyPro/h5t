use canvas::Canvas;
use crate::theme::THEME;
use ratatui::{layout::Flex, prelude::*, widgets::*};
use super::popup_area;

/// A popup to get a line of input from the user.
///
/// This widget doesn't actually handle input, it simply acts as a container for the input field.
pub struct Input<'a> {
    /// The color of the border.
    color: Color,

    /// The prompt to display as the title of the input box.
    prompt: &'a str,

    /// The text that the user has entered.
    value: &'a str,

    /// Maximum length of the input field.
    max_length: usize,
}

impl<'a> Input<'a> {
    /// Create a new [`Input`] popup with all the required fields.
    pub fn new(
        color: Color,
        prompt: &'a str,
        value: &'a str,
        max_length: usize,
    ) -> Self {
        Self {
            color,
            prompt,
            value,
            max_length,
        }
    }
}

impl Widget for Input<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // center the input
        let size = (
            // 4 includes borders and text padding
            self.prompt.len().max(self.max_length) as u16 + 4,
            3, // 2 for borders, 1 for text
        );
        let area = popup_area(area, Flex::Center, Flex::End, size, 1);

        // clear the area
        Clear.render(area, buf);
        Widget::render(
            Canvas::default()
                .background_color(THEME.background.into())
                .paint(|_| ()),
            area,
            buf,
        );

        // draw bordered box for the input field
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(self.color)
            .title(self.prompt)
            .render(area, buf);

        let text_area = area.inner(Margin::new(2, 1));

        // show input value
        Text::raw(self.value)
            .style(THEME.foreground)
            .render(text_area, buf);

        // display fake cursor
        let cursor_x = text_area.x + self.value.len() as u16;
        let cursor_y = text_area.y;
        // buf.set_.set_symbol(cursor_x, cursor_y, "▌");
        buf.cell_mut((cursor_x, cursor_y))
            .expect("cursor out of bounds")
            .set_bg(THEME.foreground.into());
    }
}
