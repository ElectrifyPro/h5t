use crate::{selectable::SelectableEnum, theme::THEME, widgets::{popup::Popup, SelectableTable}};
use ratatui::prelude::*;

/// A popup that displays a selection prompt for an enum. Like [`Multiselect`], but for a single
/// option.
///
/// This widget doesn't actually handle input, it simply acts as a container for the input.
///
/// [`Multiselect`]: super::Multiselect
pub struct Select<'a, T> {
    /// The prompt to display as the title of the input box.
    prompt: &'a str,

    /// The list of options that can be chosen.
    options: &'a [T],

    /// The selected options.
    selected: Option<&'a T>,

    /// Whether to render the widget in an active state.
    active: bool,
}

impl<'a, T> Select<'a, T> {
    /// Create a [`Select`] popup with the given options and an optional preselected option.
    #[allow(dead_code)] // NOTE: included for completeness
    pub fn with_options(
        prompt: &'a str,
        options: &'a [T],
        selected: Option<&'a T>,
        active: bool,
    ) -> Self {
        Self { prompt, options, selected, active }
    }
}

impl<'a, E: SelectableEnum + 'static> Select<'a, E> {
    /// Create a [`Select`] popup from a [`SelectableEnum`] and an optional preselected option.
    pub fn with_enum(prompt: &'a str, selected: Option<&'a E>, active: bool) -> Self {
        Self { prompt, options: E::variants(), selected, active }
    }
}

impl<T: SelectableEnum> Widget for Select<'_, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let widget = SelectableTable::<T, _, _>::with_options(
            self.options,
            |_, item: &T| if let Some(selected_t) = self.selected {
                selected_t == item
            } else {
                false
            },
            |_, _: &T| self.active,
        );

        let popup = Popup::new(THEME.foreground, Some(self.prompt), self.active, widget);
        popup.render(area, buf);
    }
}
