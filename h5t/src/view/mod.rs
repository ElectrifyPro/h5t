pub mod battle;
pub mod setup;

/// Labels used for label mode.
///
/// When interacting with a menu that requires selection of one or multiple options, label mode is
/// actitvated. In this mode, characters from this string are placed alongside each option, and the
/// user can select an option by pressing the label that corresponds with the option. A user can
/// quickly select an option without having their hands leave the keyboard.
///
/// The sequence of labels is simply the characters on a QUERTY keyboard, starting from the top-left
/// and moving down, then right. This keeps labels physically close to each other on the keyboard.
// TODO: change labels for different keyboard layouts
pub(crate) const LABELS: &str = "qazwsxedcrfvtgbyhnujmik,ol.p;/[']";
