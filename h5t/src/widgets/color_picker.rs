use crate::{theme::Rgb, widgets::popup::SizedWidget};
use ratatui::{layout::Offset, prelude::*};

/// A widget to pick an RGB color.
#[derive(Debug)]
pub struct ColorPicker {
    /// The RGB color.
    color: Rgb,

    /// The width of the gradient bars.
    gradient_width: u16,
}

impl ColorPicker {
    /// Create a new [`ColorPicker`] widget with the given selected color.
    pub fn new(color: Rgb, gradient_width: u16) -> Self {
        Self { color, gradient_width }
    }

    /// Renders the gradient bar and helper icons / text for one color channel.
    fn render_channel(
        &self,
        area: Rect,
        buf: &mut Buffer,
        y_offset: i32,
        channel_value: u8,
        make_color: impl Fn(f32) -> Color,
    ) {
        let area = area + Offset::new(0, y_offset);

        for x in 0..self.gradient_width {
            let Some(cell) = buf.cell_mut((area.x + x, area.y)) else {
                continue;
            };

            let t = x as f32 / self.gradient_width as f32;
            cell.set_bg(make_color(t));
        }

        let inv_lerp = |v: u8| v as f32 / 255.0;

        let marker_x = (inv_lerp(channel_value) * (self.gradient_width - 1) as f32) as u16;
        if let Some(cell) = buf.cell_mut((area.x + marker_x, area.y + 1)) {
            cell.set_char('⮝');
        }

        Span::raw(channel_value.to_string())
            .render(area + Offset::new(self.gradient_width as i32 + 1, 0), buf);
    }
}

impl SizedWidget for ColorPicker {
    fn width(&self) -> u16 {
        // padding between color names and gradient bar = 6
        // padding between gradient bar and text (1) + u8 number (3) = 4
        6 + self.gradient_width + 4
    }

    fn height(&self) -> u16 {
        // 1 row of padding on top, 2 per row
        7
    }
}

impl Widget for ColorPicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let name_area = Rect {
            y: area.y + 1,
            width: 5, // length of "Green"
            ..area
        };
        Text::raw("Red\n\nGreen\n\nBlue").right_aligned().render(name_area, buf);

        let remainder_area = Rect {
            x: area.x + 6, // length of "Green" + padding (1)
            ..area
        };

        // return a color channel value interpolated betweeen 0-255
        let lerp = |t: f32| (t * 255.0) as u8;

        let Rgb(r, g, b) = self.color;
        self.render_channel(remainder_area, buf, 1, r, |t| Color::Rgb(lerp(t), g, b));
        self.render_channel(remainder_area, buf, 3, g, |t| Color::Rgb(r, lerp(t), b));
        self.render_channel(remainder_area, buf, 5, b, |t| Color::Rgb(r, g, lerp(t)));
    }
}
