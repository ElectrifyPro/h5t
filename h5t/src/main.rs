pub use h5t_core::{Monster, Tracker};

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
        find("ogre").into(),
        find("tarrasque").into(),
    ]);
    for _ in 0..100 {
        println!("({}, {})", tracker.round, tracker.turn);
        tracker.next_turn();
    }
}
