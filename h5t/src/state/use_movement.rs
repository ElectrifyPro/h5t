use crate::{
    input::{AfterKey as AfterKeyInner, Charset, GetInput},
    theme::THEME,
    widgets::popup::Popup,
    Tracker,
};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::resource::{MovementSpeed, Resource};
use ratatui::{layout::{Flex, Offset}, prelude::*};
use super::AfterKey;

/// Unit-agnostic distance that stores distance in units of feet.
#[derive(Clone, Debug, Default)]
struct Distance {
    feet: i32,
}

impl Distance {
    /// Returns the contained distance in 5-foot squares.
    fn get_squares(&self) -> i32 {
        self.feet / 5
    }

    /// Returns the contained distance in feet.
    fn get_feet(&self) -> i32 {
        self.feet
    }

    /// Returns the distance spent in feet when moving through difficult terrain (costs double
    /// movement).
    fn get_difficult_terrain_feet(&self) -> i32 {
        self.feet * 2
    }

    /// Sets the contained distance in 5-foot squares.
    fn set_squares(&mut self, squares: i32) {
        self.feet = squares * 5; // 5 ft. per square
    }

    /// Sets the contained distance in feet.
    fn set_feet(&mut self, feet: i32) {
        self.feet = feet;
    }
}

/// Helper enum to indicate which input field is currently selected.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Field {
    #[default]
    Squares,
    Feet,
}

/// State for spending a combatant's movement.
#[derive(Clone, Debug, Default)]
pub struct UseMovement {
    /// Helper that gets input from the user in the unit of squares moved.
    distance_squares: GetInput<i32>,

    /// Helper that gets input from the user in the unit of feet moved.
    distance_feet: GetInput<i32>,

    /// The distance set by the user.
    distance: Distance,

    /// Indicates which of the above input fields is currently selected.
    selected: Field,

    /// Whether this movement is through difficult terrain.
    difficult_terrain: bool,
}

impl UseMovement {
    /// Create a [`UseMovement`] state with all the required fields.
    pub fn new() -> Self {
        Self {
            // movement is usually 1-2 digits
            distance_squares: GetInput::new("Movement amount", 4, Charset::Numeric)
                .suffix("squares"),
            distance_feet: GetInput::new("Movement amount", 4, Charset::Numeric)
                .active(false)
                .suffix("feet"),
            distance: Distance::default(),
            selected: Field::default(),
            difficult_terrain: false,
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let [
            distance_squares,
            distance_feet,
            difficult_terrain_toggle,
            calculation_box,
        ] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .flex(Flex::Center)
            .areas(frame.area());
        self.distance_squares.draw(frame, distance_squares);
        self.distance_feet.draw(frame, distance_feet);

        // difficult terrain toggle
        // "⦿  difficult terrain".len() = 20
        let popup = Popup::new(THEME.foreground, "", 20, 1, true);
        let block_area = popup.block_area(difficult_terrain_toggle);
        popup.render(difficult_terrain_toggle, frame.buffer_mut());

        Line::from(vec![
            Span::raw(if self.difficult_terrain { "◉" } else { "○" }),
            Span::raw("  difficult terrain"),
        ])
            .centered()
            // .style(if self.difficult_terrain { Modifier::BOLD } else { Modifier::empty() })
            .style(Modifier::BOLD)
            .render(block_area + Offset::new(0, 1), frame.buffer_mut());

        // movement calculation guide
        let movement_used = if self.difficult_terrain {
            self.distance.get_difficult_terrain_feet()
        } else {
            self.distance.get_feet()
        };

        let calculation = if self.difficult_terrain {
            format!(
                "{} ft * 2 = {} ft movement used",
                self.distance.get_feet(),
                movement_used
            )
        } else {
            format!("{} ft movement used", movement_used)
        };

        let calc_width = calculation.len() as u16;
        let calc_popup = Popup::new(THEME.foreground, "", calc_width, 1, true);
        let calc_block = calc_popup.block_area(calculation_box);

        calc_popup.render(calculation_box, frame.buffer_mut());

        Line::from(calculation)
            .centered()
            .style(Modifier::ITALIC)
            .render(calc_block + Offset::new(0, 1), frame.buffer_mut());
    }

    /// Handle a key event and apply any needed changes to the tracker.
    pub fn handle_key(&mut self, key: KeyEvent, tracker: &mut Tracker) -> AfterKey {
        let focused_input = match self.selected {
            Field::Squares => &mut self.distance_squares,
            Field::Feet => &mut self.distance_feet,
        };
        match focused_input.handle_key(key) {
            AfterKeyInner::Handled => {
                let Ok(value_a) = focused_input.get_parsed() else {
                    return AfterKey::Stay;
                };
                match self.selected {
                    Field::Squares => {
                        self.distance.set_squares(value_a);
                        self.distance_feet.set_value(self.distance.get_feet().to_string());
                    },
                    Field::Feet => {
                        self.distance.set_feet(value_a);
                        self.distance_squares.set_value(self.distance.get_squares().to_string());
                    },
                }

                AfterKey::Stay
            },
            AfterKeyInner::Submit(_) => {
                let speed_used = if self.difficult_terrain {
                    self.distance.get_difficult_terrain_feet()
                } else {
                    self.distance.get_feet()
                };
                let current_speed = tracker
                    .current_combatant_mut()
                    .resource_pool
                    .get_mut(&MovementSpeed::ID);
                *current_speed -= speed_used;
                AfterKey::Exit
            },
            AfterKeyInner::Cancel => AfterKey::Exit,
            AfterKeyInner::Forward(event) => {
                match event.code {
                    KeyCode::Char('q') => {
                        self.selected = Field::Squares;
                        self.distance_squares.set_active(true);
                        self.distance_feet.set_active(false);
                    },
                    KeyCode::Char('a') => {
                        self.selected = Field::Feet;
                        self.distance_squares.set_active(false);
                        self.distance_feet.set_active(true);
                    },
                    KeyCode::Char('z') => {
                        self.difficult_terrain = !self.difficult_terrain;
                    },
                    _ => (),
                }
                AfterKey::Stay
            },
        }
    }
}
