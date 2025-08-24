/// An RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// Mix the current color with another color by taking the average of their RGB components.
    pub fn mix(self, other: Rgb) -> Rgb {
        Rgb(
            self.0.midpoint(other.0),
            self.1.midpoint(other.1),
            self.2.midpoint(other.2),
        )
    }
}

impl From<Rgb> for ratatui::style::Color {
    fn from(rgb: Rgb) -> Self {
        ratatui::style::Color::Rgb(rgb.0, rgb.1, rgb.2)
    }
}

impl From<Rgb> for ratatui::style::Style {
    fn from(rgb: Rgb) -> Self {
        ratatui::style::Style::default().fg(rgb.into())
    }
}

/// A theme specifying the colors used in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub background: Rgb,
    pub foreground: Rgb,
    pub primary: Rgb,
    pub secondary: Rgb,
    pub select: Rgb,
    pub accent: Rgb,
    pub error: Rgb,
    pub warning: Rgb,

    // domain-specific colors

    pub action: Rgb,
    pub bonus_action: Rgb,
    pub reaction: Rgb,
    pub dead: Rgb,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::new()
    }
}

impl Theme {
    /// Returns the default theme.
    pub const fn new() -> Self {
        Theme {
            background: Rgb(26, 27, 38), // indigo
            foreground: Rgb(192, 202, 245), // light blue
            primary: Rgb(0, 48, 130), // dark blue
            secondary: Rgb(65, 72, 104), // blue gray
            select: Rgb(128, 85, 0), // dark yellow
            accent: Rgb(255, 165, 0),
            error: Rgb(247, 118, 142), // pastel red
            warning: Rgb(224, 175, 104), // pastel yellow

            action: Rgb(158, 206, 106), // lime green
            bonus_action: Rgb(255, 165, 0), // gold
            reaction: Rgb(187, 154, 247), // pastel purple
            dead: Rgb(100, 0, 0), // dark red
        }
    }

    /// Return a dimmed version of the theme, obtained by mixing each color with the background
    /// color.
    pub fn dim(self) -> Self {
        let background = self.background;
        Theme {
            background,
            foreground: self.foreground.mix(background),
            primary: self.primary.mix(background),
            secondary: self.secondary.mix(background),
            select: self.select.mix(background),
            accent: self.accent.mix(background),
            error: self.error.mix(background),
            warning: self.warning.mix(background),

            action: self.action.mix(background),
            bonus_action: self.bonus_action.mix(background),
            reaction: self.reaction.mix(background),
            dead: self.dead.mix(background),
        }
    }
}

/// The default theme for the UI.
pub static THEME: Theme = Theme {
    background: Rgb(26, 27, 38), // indigo
    foreground: Rgb(192, 202, 245), // light blue
    primary: Rgb(0, 48, 130), // dark blue
    secondary: Rgb(65, 72, 104), // blue gray
    select: Rgb(128, 85, 0), // dark yellow
    accent: Rgb(255, 165, 0),
    error: Rgb(247, 118, 142), // pastel red
    warning: Rgb(224, 175, 104), // pastel yellow

    action: Rgb(158, 206, 106), // lime green
    bonus_action: Rgb(255, 165, 0), // gold
    reaction: Rgb(187, 154, 247), // pastel purple
    dead: Rgb(100, 0, 0), // dark red
};
