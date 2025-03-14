mod monster;

use crossterm::event::{read, Event, KeyCode};
use h5t_core::{Combatant, Monster, Tracker};
use monster::MonsterCard;

fn main() {
    // NOTE: monster JSON data provided courtesy of https://www.dnd5eapi.co/
    let file = std::fs::File::open("data/monsters.json").unwrap();
    let monsters = serde_json::from_reader::<_, Vec<Monster>>(file).unwrap();
    // println!("{:#?}", monsters);

    let mut tracker = Tracker::new(monsters.into_iter().map(Combatant::Monster).collect::<Vec<_>>());

    let mut terminal = ratatui::init();

    for _ in 0..tracker.combatants.len() {
        // println!("({}, {})", tracker.round, tracker.turn);
        // println!("{:#?}", tracker.current_combatant());
        // print a nice card
        terminal.draw(|frame| {
            let area = frame.area();
            let Combatant::Monster(monster) = tracker.current_combatant();
            frame.render_widget(MonsterCard::new(monster), area);
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
