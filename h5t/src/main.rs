mod monster;

use crossterm::event::read;
use h5t_core::{Combatant, Monster, Tracker};
use monster::MonsterCard;

fn main() {
    // NOTE: monster JSON data provided courtesy of https://www.dnd5eapi.co/
    let file = std::fs::File::open("data/monsters.json").unwrap();
    let monsters = serde_json::from_reader::<_, Vec<Monster>>(file).unwrap();
    // println!("{:#?}", monsters);

    let find = |idx: &str| -> Monster {
        monsters.iter().find(|monster| monster.index == idx).unwrap().clone()
    };

    let mut tracker = Tracker::new(vec![
        find("goblin").into(),
        find("boar").into(),
        find("tarrasque").into(),
    ]);

    let mut terminal = ratatui::init();

    for _ in 0..3 {
        // println!("({}, {})", tracker.round, tracker.turn);
        // println!("{:#?}", tracker.current_combatant());
        // print a nice card
        terminal.draw(|frame| {
            let area = frame.area();
            let Combatant::Monster(monster) = tracker.current_combatant();
            frame.render_widget(MonsterCard::new(monster), area);
        }).unwrap();
        read().unwrap();
        tracker.next_turn();
    }

    ratatui::restore();
}
