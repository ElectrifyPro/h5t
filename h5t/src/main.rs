mod input;
mod selectable;
mod state;
mod theme;
mod ui;
mod widgets;

use h5t_core::{CombatantKind, Monster, Spell, Tracker};
use ui::Ui;

fn main() {
    // NOTE: monster and spell JSON data provided courtesy of https://www.dnd5eapi.co/
    let file = std::fs::File::open("data/monsters.json").unwrap();
    let monsters = serde_json::from_reader::<_, Vec<Monster>>(file).unwrap();
    let file = std::fs::File::open("data/spells.json").unwrap();
    let spells = serde_json::from_reader::<_, Vec<Spell>>(file).unwrap();

    let mut tracker = Ui::new(
        ratatui::init(),
        Tracker::new(monsters
            .into_iter()
            .map(|m| CombatantKind::Monster(m).into())
            .collect::<Vec<_>>()),
    );

    tracker.run();
}
