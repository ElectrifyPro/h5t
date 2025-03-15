mod selectable;
mod ui;
mod widgets;

use crossterm::event::{read, Event, KeyCode};
use h5t_core::{CombatantKind, Condition, Monster, Tracker};
use ui::Ui;

fn main() {
    // NOTE: monster JSON data provided courtesy of https://www.dnd5eapi.co/
    let file = std::fs::File::open("data/monsters.json").unwrap();
    let monsters = serde_json::from_reader::<_, Vec<Monster>>(file).unwrap();
    // println!("{:#?}", monsters);

    let mut tracker = Ui::new(
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
                KeyCode::Char('c') => {
                    // apply condition
                    // let selected = tracker.enter_label_mode();
                    let conditions = tracker.multi_select_enum::<15, Condition>("Select condition(s)");
                    panic!("conditions: {:?}", conditions);
                },
                KeyCode::Char('d') => {
                    // TEST: choose and damage a combatant
                    let selected = tracker.enter_label_mode();
                    let value = tracker.get_value::<i32>("Damage amount").unwrap();
                    for combatant_idx in selected {
                        let combatant = &mut tracker.tracker.combatants[combatant_idx];
                        combatant.damage(value);
                    }
                    tracker.label_state = None;
                    continue;
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
                KeyCode::Char('s') => {
                    tracker.toggle_stat_block();
                    continue;
                },
                KeyCode::Char('q') => break,
                _ => (),
            }
        }
        tracker.next_turn();
    }
}
