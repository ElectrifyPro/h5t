use crate::{selectable::SelectableEnum, theme::THEME, widgets::{SelectableTable, popup::Popup}};
use ratatui::prelude::*;
use std::collections::HashSet;

/// A popup that displays a multi-select prompt for an enum. Like [`Select`], but for multiple
/// options.
///
/// This widget doesn't actually handle input, it simply acts as a container for the input.
///
/// [`Select`]: super::Select
pub struct Multiselect<'a, T> {
    /// The prompt to display as the title of the input box.
    prompt: &'a str,

    /// The list of options that can be chosen.
    options: &'a [T],

    /// The selected options.
    selected: &'a HashSet<T>,

    /// Whether to render the widget in an active state.
    active: bool,
}

impl<'a, T> Multiselect<'a, T> {
    /// Create a [`Multiselect`] popup with the given options and selection state.
    pub fn with_options(
        prompt: &'a str,
        options: &'a [T],
        selected: &'a HashSet<T>,
        active: bool,
    ) -> Self {
        Self { prompt, options, selected, active }
    }
}

impl<'a, E: SelectableEnum + 'static> Multiselect<'a, E> {
    /// Create a [`Multiselect`] popup from a [`SelectableEnum`] and selection state.
    pub fn with_enum(prompt: &'a str, selected: &'a HashSet<E>, active: bool) -> Self {
        Self { prompt, options: E::variants(), selected, active }
    }
}

impl<T: Eq + std::hash::Hash + std::fmt::Display> Widget for Multiselect<'_, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let widget = SelectableTable::<T, _, _>::with_options(
            self.options,
            |_, item: &T| self.selected.contains(item),
            |_, _: &T| self.active,
        );
        let prompt = format!("{} ({}/{})", self.prompt, self.selected.len(), self.options.len());
        let popup = Popup::new(THEME.foreground, Some(&prompt), self.active, widget);
        popup.render(area, buf);
    }
}
