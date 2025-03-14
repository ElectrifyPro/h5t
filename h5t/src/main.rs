mod monster;
mod tracker;
mod ui_tracker;

use crossterm::event::{read, Event, KeyCode};
use h5t_core::{CombatantKind, Monster, Tracker};
use ui_tracker::UiTracker;

fn main() {
    // NOTE: monster JSON data provided courtesy of https://www.dnd5eapi.co/
    let file = std::fs::File::open("data/monsters.json").unwrap();
    let monsters = serde_json::from_reader::<_, Vec<Monster>>(file).unwrap();
    // println!("{:#?}", monsters);

    let mut tracker = UiTracker::new(
        ratatui::init(),
        Tracker::new(monsters
            .into_iter()
            .map(|m| CombatantKind::Monster(m).into())
            .collect::<Vec<_>>()),
    );

    for _ in 0..tracker.combatants.len() {
        tracker.draw().unwrap();
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Char('d') => {
                    // TEST: choose and damage a combatant
                    let selected = tracker.enter_label_mode();
                    panic!("{:#?}", selected);
                },
                KeyCode::Char('a') => {
                    tracker.use_action();
                    continue;
                },
                KeyCode::Char('b') => {
                    tracker.use_bonus_action();
                    continue;
                },
                KeyCode::Char('r') => {
                    tracker.use_reaction();
                    continue;
                },
                KeyCode::Char('q') => break,
                _ => (),
            }
        }
        tracker.next_turn();
    }
}
