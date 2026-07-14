use crate::{selectable::SelectableEnum, theme::THEME, view::LABELS, widgets::popup::SizedWidget};
use ratatui::{prelude::*, widgets::*};

/// FIXME: when we attempt to write a closure for use as the `render_fn` in `SelectableTable`, Rust
/// has trouble inferring that `Text` is supposed to borrow from `T`. but if you wrap it in this,
/// it'll work
///
/// you can do it compiler :) :)
pub(crate) fn infer<T, F>(f: F) -> F
where F: Fn(usize, &T) -> Text,
{
    f
}

/// A table widget that displays a list of options, and supports highlighting any number of options
/// as selected.
pub struct SelectableTable<'a, T, S, A, F> {
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

    /// A function that renders the option as a [`Text`] widget.
    ///
    /// NOTE: If the returned [`Text`] is styled (e.g. with custom colors), it will override the
    /// styles set by [`SelectableTable`], such as the dimmed color when the [`Text`] is inactive.
    ///
    /// It takes two parameters: a `usize` representing the index of the given option in the parent
    /// list, and a reference `&T` to the option. Either/or can be used as needed.
    render_fn: F,
}

impl<'a, T, S, A, F> SelectableTable<'a, T, S, A, F> {
    /// Create a [`SelectableTable`] with the given options.
    pub fn with_options(options: &'a [T], selected_fn: S, active_fn: A, render_fn: F) -> Self {
        Self { options, selected_fn, active_fn, render_fn }
    }
}

impl<E: SelectableEnum, S, A, F> SelectableTable<'static, E, S, A, F> {
    /// Create a [`SelectableTable`] from a [`SelectableEnum`].
    #[allow(dead_code)] // NOTE: included for completeness
    pub fn with_enum(selected_fn: S, active_fn: A, render_fn: F) -> Self {
        Self { options: E::variants(), selected_fn, active_fn, render_fn }
    }
}

impl<'a, T, S, A, F> SizedWidget for SelectableTable<'a, T, S, A, F>
where
    S: Fn(usize, &T) -> bool,
    A: Fn(usize, &T) -> bool,
    F: Fn(usize, &T) -> Text,
{
    fn width(&self) -> u16 {
        let Some(longest_variant_length) = self.options
            .iter()
            .enumerate()
            .map(|(idx, v)| (self.render_fn)(idx, v).width())
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

impl<'a, T, S, A, F> Widget for SelectableTable<'a, T, S, A, F>
where
    S: Fn(usize, &T) -> bool,
    A: Fn(usize, &T) -> bool,
    F: Fn(usize, &T) -> Text,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let table = Table::new(
            LABELS.into_iter()
                .zip(self.options)
                .enumerate()
                .map(|(idx, (label, variant))| {
                    let theme = if (self.active_fn)(idx, variant) {
                        THEME
                    } else {
                        THEME.dim()
                    };
                    let mut style = Style::default()
                        .fg(theme.foreground.into())
                        .bg(theme.background.into());

                    if (self.selected_fn)(idx, variant) {
                        style = style.bold().bg(theme.select.into());
                    }

                    Row::new(vec![
                        Text::styled(label.to_string(), Modifier::BOLD),
                        (self.render_fn)(idx, variant),
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
