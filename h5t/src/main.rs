mod monster;
mod tracker;

use crossterm::event::{read, Event, KeyCode};
use h5t_core::{CombatantKind, Monster, Tracker};

fn main() {
    // NOTE: monster JSON data provided courtesy of https://www.dnd5eapi.co/
    let file = std::fs::File::open("data/monsters.json").unwrap();
    let monsters = serde_json::from_reader::<_, Vec<Monster>>(file).unwrap();
    // println!("{:#?}", monsters);

    let mut tracker = Tracker::new(monsters
        .into_iter()
        .map(|m| CombatantKind::Monster(m).into())
        .collect::<Vec<_>>());

    let mut terminal = ratatui::init();

    for _ in 0..tracker.combatants.len() {
        terminal.draw(|frame| {
            let area = frame.area();
            // print a nice card
            // let combatant = tracker.current_combatant();
            // let CombatantKind::Monster(monster) = &combatant.kind;
            // frame.render_widget(monster::MonsterCard::new(monster), area);

            // print tracker
            frame.render_widget(tracker::TrackerWidget::new(&tracker), area);
        }).unwrap();
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Char('q') => break,
                _ => (),
            }
        }
        tracker.next_turn();
    }

    ratatui::restore();
}
