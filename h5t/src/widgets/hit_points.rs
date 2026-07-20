use crate::theme::THEME;
use h5t_core::{Combatant, Health};
use ratatui::prelude::*;

/// A widget to display a creature's health state, changing color based on the current hit points.
#[derive(Debug)]
pub struct HitPoints {
    /// The creature's current health state.
    pub current: Health,

    /// The creature's maximum hit points.
    pub max: i32,
}

impl HitPoints {
    /// Create a new [`HitPoints`] widget from a [`Combatant`].
    pub fn new(combatant: &Combatant) -> Self {
        Self {
            current: combatant.health,
            max: combatant.max_hit_points(),
        }
    }

    /// Creates a [`Line`] widget containing the hit points display.
    pub fn line(&self) -> Line<'static> {
        // TODO: copied straight from widgets/tracker.rs
        fn fmt_action(label: &str, count: i32) -> String {
            match count {
                ..=0 => "   ".to_string(),
                1..=3 => format!("{:<3}", label.repeat(count as usize)),
                4.. => format!("{}x{}", label, count),
            }
        }

        match self.current {
            Health::Hp(hp) => {
                let hp = hp.get();
                let hp_color = Color::Rgb(
                    (255.0 - hp as f32 / self.max as f32 * 255.0) as u8,
                    (hp as f32 / self.max as f32 * 255.0) as u8,
                    0,
                );

                Line::from(vec![
                    Span::styled(format!("{hp}"), hp_color),
                    Span::raw(format!(" / {}", self.max)),
                ])
            },
            Health::Downed(counts) => Line::from(vec![
                Span::styled(fmt_action("P", counts.successes as i32), THEME.success),
                Span::styled("|", THEME.foreground),
                Span::styled(fmt_action("F", counts.failures as i32), THEME.error),
            ]),
            Health::Stabilized => Line::from(vec![
                Span::styled("0", THEME.warning),
                Span::raw(format!(" / {}", self.max)),
            ]),
            Health::Dead => Line::from(vec![
                Span::raw("💀"),
                Span::raw(format!(" / {}", self.max)),
            ]),
        }
    }
}

impl Widget for HitPoints {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.line().render(area, buf);
    }
}
