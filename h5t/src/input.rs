use crate::widgets::popup::Input;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use std::str::FromStr;

/// The result of receiving a key event with [`GetInput`].
pub enum AfterKey<T> {
    /// The key event was successfully handled, and we are still in the input state.
    Handled,

    /// The user submitted the given valid input (i.e. pressed `Enter`).
    Submit(T),

    /// The user cancelled the input (i.e. pressed `Esc`).
    Cancel,

    /// The key event has no use in the input state and will be forwarded to the caller of
    /// [`GetInput::handle_key`].
    Forward(KeyEvent),
}

/// Helper type that displays an [`Input`] widget, accepts user input, validates it, and returns
/// the result.
pub struct GetInput<T> {
    /// The prompt to display as the title of the input box.
    prompt: String,

    /// The value of the input field.
    pub value: String,

    /// Maximum length of the input field.
    max_length: usize,

    /// Whether the input has been touched, i.e. the user has ever entered a character in it, even
    /// if they have since deleted it.
    ///
    /// This is used to mark the input "valid" (i.e. no red error border) if the user has not
    /// entered any characters yet.
    touched: bool,

    _marker: std::marker::PhantomData<T>,
}

// manually implement `Clone` since `#[derive(Clone)]`ing it adds an unecessary `T: Clone` bound
impl<T> Clone for GetInput<T> {
    fn clone(&self) -> Self {
        Self {
            prompt: self.prompt.clone(),
            value: self.value.clone(),
            max_length: self.max_length,
            touched: self.touched,
            _marker: std::marker::PhantomData,
        }
    }
}

// same reasoning for `Debug` and `Default`
impl<T> std::fmt::Debug for GetInput<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetInput")
            .field("prompt", &self.prompt)
            .field("value", &self.value)
            .field("max_length", &self.max_length)
            .field("touched", &self.touched)
            .finish()
    }
}

impl<T> Default for GetInput<T> {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            value: String::new(),
            max_length: 0,
            touched: false,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: FromStr> GetInput<T> {
    /// Create a new [`GetInput`] helper with all the required fields.
    pub fn new(prompt: impl Into<String>, max_length: usize) -> Self {
        Self {
            prompt: prompt.into(),
            value: String::new(),
            max_length,
            touched: false,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the color of the input field based on the validity of the input.
    pub fn color(&self) -> Color {
        if self.value.len() >= self.max_length {
            Color::Yellow
        } else if T::from_str(&self.value).is_ok() || !self.touched {
            Color::Reset
        } else {
            Color::Red
        }
    }

    /// Draw the input widget to the given area.
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Input::new(
            self.color(),
            &self.prompt,
            &self.value,
            self.max_length,
        ), area)
    }

    /// Handle a key event and update the input value.
    pub fn handle_key(&mut self, key: KeyEvent) -> AfterKey<T> {
        match key.code {
            KeyCode::Enter => if let Ok(value) = T::from_str(&self.value) {
                return AfterKey::Submit(value);

            } else {
                // handled: input will become red
                self.touched = true;
            },
            KeyCode::Char(c) => {
                if self.value.len() < self.max_length {
                    self.value.push(c);
                }

                // handled; input will become yellow or green
                self.touched = true;
            },
            KeyCode::Backspace => {
                self.value.pop();
            },
            KeyCode::Esc => return AfterKey::Cancel,
            _ => return AfterKey::Forward(key),
        }

        AfterKey::Handled
    }
}
