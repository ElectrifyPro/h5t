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

/// A widget / something that can be rendered that knows its exact size at render time.
pub trait SizedWidget: Widget {
    /// Returns the widget's width.
    fn width(&self) -> u16;

    /// Returns the widget's height.
    fn height(&self) -> u16;

    /// Returns the widget's width and height.
    fn size(&self) -> (u16, u16) {
        (self.width(), self.height())
    }
}

impl SizedWidget for Line<'_> {
    fn width(&self) -> u16 {
        // how convenient ;)
        // TODO: same as `self.width() as u16`?
        Line::width(self) as u16
    }

    fn height(&self) -> u16 {
        1
    }
}

/// Computes the [`Rect`] within the `target_area` needed to render a popup of size `size`, aligned
/// within the `target_area` `horizontal`ly and `vertical`ly as specified.
pub(crate) fn popup_area(
    target_area: Rect,
    horizontal: HorizontalAlignment,
    vertical: VerticalAlignment,
    size: (u16, u16),
) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(size.0)])
        .flex(match horizontal {
            HorizontalAlignment::Left => Flex::Start,
            HorizontalAlignment::Center => Flex::Center,
            HorizontalAlignment::Right => Flex::End,
        })
        .areas(target_area);
    let [area] = Layout::vertical([Constraint::Length(size.1)])
        .flex(match vertical {
            VerticalAlignment::Top => Flex::Start,
            VerticalAlignment::Center => Flex::Center,
            VerticalAlignment::Bottom => Flex::End,
        })
        .areas(area);

    area
}

/// A widget that renders the smallest possible [`Block`] around its content.
///
/// This can leave negative space if the target area of a [`Popup`] is significantly large. In this
/// case, the [`Popup`]'s alignment can be used to position the [`Popup`] within the target area.
pub struct Popup<'a, W> {
    /// The color of the border.
    color: Rgb,

    /// The prompt to display as the title of the popup.
    prompt: Option<&'a str>,

    /// Whether to render the popup in an active state.
    active: bool,

    /// Horizontal alignment within the popup's target area. This has no effect if the target
    /// area's size matches the size of the computed popup.
    horizontal_alignment: HorizontalAlignment,

    /// Vertical alignment within the popup's target area. This has no effect if the target area's
    /// size matches the size of the computed popup.
    vertical_alignment: VerticalAlignment,

    /// The renderable widget contained within the popup.
    inner_widget: W,
}

impl<'a, W> Popup<'a, W> {
    /// Create a new [`Popup`] with all the required fields, centered in its target area.
    pub fn new(
        color: Rgb,
        prompt: Option<&'a str>,
        active: bool,
        inner_widget: W,
    ) -> Self {
        Self {
            color,
            prompt,
            active,
            horizontal_alignment: HorizontalAlignment::Center,
            vertical_alignment: VerticalAlignment::Center,
            inner_widget,
        }
    }
}

impl<W: SizedWidget> Widget for Popup<'_, W> {
    fn render(self, target_area: Rect, buf: &mut Buffer) {
        let inner_widget_size = self.inner_widget.size();
        let prompt = self.prompt.unwrap_or("");

        let block_width = {
            // top left border (1) + top right border (1) = +2
            let width_prompt_only = 2 + prompt.len() as u16;
            // left border (1) + left padding + (1) + right padding (1) + right border (1) (1) = +4
            let width_content_only = 4 + inner_widget_size.0;
            width_prompt_only.max(width_content_only)
        };
        let block_area = popup_area(
            target_area,
            self.horizontal_alignment,
            self.vertical_alignment,
            // top (1) + bottom border (1) = +2
            (block_width, 2 + inner_widget_size.1),
        );
        let content_area = block_area.inner(Margin::new(2, 1));

        // clear the area
        Clear.render(block_area, buf);
        Widget::render(
            Canvas::default()
                .background_color(THEME.background.into())
                .paint(|_| ()),
            block_area,
            buf,
        );

        self.inner_widget.render(content_area, buf);

        let color = if self.active {
            self.color
        } else {
            self.color.mix(THEME.background)
        };

        let mut block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(color);

        if let Some(prompt) = self.prompt {
            block = block.title(prompt)
        };

        block.render(block_area, buf);
    }
}
