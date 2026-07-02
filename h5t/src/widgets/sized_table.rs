use crate::widgets::popup::SizedWidget;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

/// A wrapper around a [`Table`] that has a fixed width and height and implements [`SizedWidget`].
/// The exact widths of each column must be given.
pub struct SizedTable<'a> {
    /// The inner [`Table`].
    table: Table<'a>,

    /// Whether a header was added.
    has_header: bool,

    /// The number of rows.
    num_rows: usize,

    /// The widths of the columns, each representing a [`Constraint::Length`].
    widths: Vec<u16>,

    /// The amount of space between each column.
    column_spacing: u16,
}

impl<'a> SizedTable<'a> {
    /// Create a [`SizedTable`] with all the required fields.
    pub fn new<R, W>(rows: R, column_widths: W, column_spacing: u16) -> Self
    where
        R: IntoIterator,
        R::Item: Into<Row<'a>>,
        R::IntoIter: ExactSizeIterator,
        W: IntoIterator<Item = u16>,
    {
        let row_iter = rows.into_iter();
        let num_rows = row_iter.len();
        let widths: Vec<_> = column_widths.into_iter().collect();

        Self {
            table: Table::new(row_iter, widths.iter().copied())
                .column_spacing(column_spacing),
            has_header: false,
            num_rows,
            widths,
            column_spacing,
        }
    }

    /// Sets the header row.
    pub fn header(mut self, header: Row<'a>) -> Self {
        self.table = self.table.header(header);
        self.has_header = true;
        self
    }
}

impl SizedWidget for SizedTable<'_> {
    fn width(&self) -> u16 {
        self.widths.iter()
            .copied()
            .intersperse(self.column_spacing)
            .sum()
    }

    fn height(&self) -> u16 {
        self.num_rows as u16
            + if self.has_header { 1 } else { 0 }
    }
}

impl Widget for SizedTable<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(self.table, area, buf)
    }
}
