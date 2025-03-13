pub mod monster;

pub use monster::Monster;

/// A combatant in the initiative tracker.
///
/// Combatants can include player characters, monsters, NPCs, etc.
pub enum Combatant {
    /// Pre-made monster.
    Monster(Monster),
}

/// The core initiative tracker.
///
/// It handles the order of play and tracks every important detail, such as the current turn,
/// conditions on each combatant, actions taken, etc.
pub struct Tracker {
    /// The current turn.
    pub turn: usize,

    /// The current round, starting at 0 (to mean the first round).
    pub round: usize,

    /// The list of combatants.
    pub combatants: Vec<Combatant>,
}

fn main() {
    // NOTE: monster JSON data provided courtesy of https://www.dnd5eapi.co/
    let file = std::fs::File::open("data/monsters.json").unwrap();
    let monsters = serde_json::from_reader::<_, Vec<Monster>>(file).unwrap();
    println!("{:#?}", monsters);
}
