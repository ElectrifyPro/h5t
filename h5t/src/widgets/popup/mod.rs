//! Widgets that pop up and cover the screen.

pub mod input;
pub mod multiselect;
pub mod select;

pub use input::Input;
pub use multiselect::Multiselect;
pub use select::Select;
use ratatui::{layout::Flex, prelude::*};

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
