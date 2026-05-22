use crate::{theme::THEME, ui::LABELS, widgets::popup::Popup, Tracker};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::Action;
use ratatui::{prelude::*, widgets::*};
use super::AfterKey;

/// State for choosing an action to spend an action point on.
#[derive(Clone, Debug, Default)]
pub struct SelectAction {
    /// The actions to select from.
    actions: Vec<Action>,

    /// Index of the selected action.
    selected: Option<usize>,
}

impl SelectAction {
    /// Create an [`SelectAction`] state with the given combatants.
    pub fn new(actions: Vec<Action>) -> Self {
        Self {
            actions,
            selected: None,
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        // TODO: copied from select widget
        let area = frame.area();
        let buf = frame.buffer_mut();
        let content_width = 2 + self.actions.iter() // +2 for the labels
            .map(|v| v.name.len())
            .max()
            .unwrap_or(0) as u16;
        let popup = Popup::new(THEME.foreground, "Select action", content_width, self.actions.len() as u16, true);
        let block_area = popup.block_area(area);
        popup.render(area, buf);

        let theme = THEME;
        let widget = Table::new(
            LABELS.chars()
                .zip(&self.actions)
                .enumerate()
                .map(|(idx, (label, option))| {
                    let mut style = Style::default()
                        .fg(theme.foreground.into());

                    if let Some(selected_idx) = self.selected && selected_idx == idx {
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
            .block(Block::new().padding(Padding::symmetric(2, 1)));

        Widget::render(widget, block_area, buf);
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        match key.code {
            KeyCode::Esc => AfterKey::Exit,
            KeyCode::Enter => if let Some(idx) = self.selected {
                tracker.use_action(&self.actions[idx]);
                AfterKey::Exit
            } else {
                AfterKey::Stay
            },
            KeyCode::Char(label) => {
                let Some(idx) = LABELS.chars().position(|ch| ch == label) else {
                    return AfterKey::Stay;
                };

                if idx < self.actions.len() {
                    self.selected = Some(idx);
                }

                AfterKey::Stay
            },
            _ => AfterKey::Stay,
        }
    }
}
