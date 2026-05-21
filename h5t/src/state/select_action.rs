use canvas::Canvas;
use crate::{theme::THEME, ui::LABELS, widgets::popup::popup_area, Tracker};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::Action;
use ratatui::{layout::Flex, prelude::*, widgets::*};
use super::AfterKey;

/// State for choosing an action to spend an action point on.
#[derive(Clone, Debug, Default)]
pub struct SelectAction {
    /// The actions to select from.
    actions: Vec<Action>,

    /// Index of the selected action.
    selected: usize,
}

impl SelectAction {
    /// Create an [`SelectAction`] state with the given combatants.
    pub fn new(actions: Vec<Action>) -> Self {
        Self {
            actions,
            selected: 0,
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        // self.input.draw(frame, frame.area());
        // TODO: copied from select widget

        let prompt = "Select action";
        let area = frame.area();
        // center widget
        // 4 for borders and text padding, 2 for space for labels
        let content_width = 4 + 2 + self.actions.iter()
            .map(|v| v.name.len())
            .max()
            .unwrap_or(0) as u16;
        let size = (
            content_width.max(prompt.len() as u16 + 2),
            // 2 for top and bottom border
            2 + self.actions.len() as u16,
        );
        let area = popup_area(area, Flex::Center, Flex::Center, size, 0);

        // clear the area
        let buf = frame.buffer_mut();
        Clear.render(area, buf);
        Widget::render(
            Canvas::default()
                .background_color(THEME.background.into())
                .paint(|_| ()),
            area,
            buf,
        );

        let theme = THEME;
        let widget = Table::new(
            LABELS.chars()
                .zip(&self.actions)
                .enumerate()
                .map(|(idx, (label, option))| {
                    let mut style = Style::default()
                        .fg(theme.foreground.into());

                    if self.selected == idx {
                        style = style.bold().bg(theme.select.into());
                    }

                    Row::new(vec![
                        Text::styled(label.to_string(), Modifier::BOLD),
                        Text::raw(&option.name),
                    ]).style(style)
                }),
            [
                Constraint::Length(1),
                Constraint::Fill(1),
            ],
        )
            .block(Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(theme.foreground)
                .title(prompt)
                .padding(Padding::symmetric(1, 0)));

        Widget::render(widget, area, buf);
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        match key.code {
            KeyCode::Esc => AfterKey::Exit,
            KeyCode::Enter => {
                tracker.use_action(&self.actions[self.selected]);
                AfterKey::Exit
            },
            KeyCode::Char(label) => {
                let Some(idx) = LABELS.chars().position(|ch| ch == label) else {
                    return AfterKey::Stay;
                };

                if self.selected < self.actions.len() {
                    self.selected = idx;
                }

                AfterKey::Stay
            },
            _ => AfterKey::Stay,
        }
    }
}
