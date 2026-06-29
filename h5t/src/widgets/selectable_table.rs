use crate::{selectable::SelectableEnum, theme::THEME, view::LABELS, widgets::popup::SizedWidget};
use std::fmt::Display;
use ratatui::{prelude::*, widgets::*};

/// A table widget that displays a list of options, and supports highlighting any number of options
/// as selected.
pub struct SelectableTable<'a, T, S, A> {
    /// The list of options to show.
    options: &'a [T],

    /// A function that returns true if the given option is selected.
    ///
    /// It takes two parameters: a `usize` representing the index of the given option in the parent
    /// list, and a reference `&T` to the option. Depending on the selection state, either/or can be
    /// used to determine selection state equality.
    selected_fn: S,

    /// A function that returns true if the given option is active and can be selected at all. If an
    /// option is inactive, it is greyed out.
    ///
    /// It takes two parameters: a `usize` representing the index of the given option in the parent
    /// list, and a reference `&T` to the option. Either/or can be used as needed.
    active_fn: A,
}

impl<'a, T, S, A> SelectableTable<'a, T, S, A> {
    /// Create a [`SelectableTable`] with the given options.
    pub fn with_options(options: &'a [T], selected_fn: S, active_fn: A) -> Self {
        Self { options, selected_fn, active_fn }
    }
}

impl<E: SelectableEnum, S, A> SelectableTable<'static, E, S, A> {
    /// Create a [`SelectableTable`] from a [`SelectableEnum`].
    #[allow(dead_code)] // NOTE: included for completeness
    pub fn with_enum(selected_fn: S, active_fn: A) -> Self {
        Self { options: E::variants(), selected_fn, active_fn }
    }
}

impl<'a, T, S, A> SizedWidget for SelectableTable<'a, T, S, A>
where
    T: Display,
    S: Fn(usize, &T) -> bool,
    A: Fn(usize, &T) -> bool,
{
    fn width(&self) -> u16 {
        let Some(longest_variant_length) = self.options
            .iter()
            .map(|v| v.to_string().len())
            .max() else {
            // this makes no sense and should also not be possible
            return 0;
        };
        // 2 for the labels column and the spacing
        2 + longest_variant_length as u16
    }

    fn height(&self) -> u16 {
        self.options.len() as u16
    }
}

impl<'a, T, S, A> Widget for SelectableTable<'a, T, S, A>
where
    T: Display,
    S: Fn(usize, &T) -> bool,
    A: Fn(usize, &T) -> bool,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let table = Table::new(
            LABELS.chars()
                .zip(self.options)
                .enumerate()
                .map(|(idx, (label, variant))| {
                    let theme = if (self.active_fn)(idx, variant) {
                        THEME
                    } else {
                        THEME.dim()
                    };
                    let mut style = Style::default()
                        .fg(theme.foreground.into());

                    if (self.selected_fn)(idx, variant) {
                        style = style.bold().bg(theme.select.into());
                    }

                    Row::new(vec![
                        Text::styled(label.to_string(), Modifier::BOLD),
                        Text::from(variant.to_string()),
                    ]).style(style)
                }),
            [
                Constraint::Length(1),
                Constraint::Fill(1),
            ],
        );
        Widget::render(table, area, buf);
    }
}
