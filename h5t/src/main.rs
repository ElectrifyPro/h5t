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

    let mut combatants = monsters
        .into_iter()
        .map(|m| CombatantKind::Monster(m).into())
        .collect::<Vec<_>>();

    use h5t_core::{ability::{Proficiencies, Skill}, monster::ArmorClass, resource::ResourcePool, speed::Speed, Ability, Character, Combatant};
    use std::collections::HashMap;
    let pcs = vec![
        Combatant {
            kind: CombatantKind::Character(Character {
                index: String::from("astarion"),
                name: String::from("Astarion"),
                scores: Ability {
                    strength: 8,
                    dexterity: 17,
                    constitution: 14,
                    intelligence: 13,
                    wisdom: 13,
                    charisma: 10,
                },
                armor_class: ArmorClass {
                    value: 13,
                    ..Default::default()
                },
                hit_points: 10,
                damage_vulnerabilities: HashMap::new(),
                damage_resistances: HashMap::new(),
                damage_immunities: HashMap::new(),
                speed: Speed {
                    walk: Some(30),
                    ..Default::default()
                },
                proficiencies: Proficiencies {
                    skills: Skill {
                        acrobatics: Some(5),
                        deception: Some(2),
                        perception: Some(3),
                        performance: Some(2),
                        persuasion: Some(2),
                        sleight_of_hand: Some(7),
                        stealth: Some(7),
                        ..Default::default()
                    },
                    saving_throws: Ability {
                        dexterity: Some(5),
                        intelligence: Some(3),
                        ..Default::default()
                    },
                },
                level: 1,
            }),
            conditions: vec![],
            hit_points: 32,
            resource_pool: ResourcePool::default(),
        },
    ];
    combatants.splice(0..0, pcs);

    let mut tracker = Ui::new(
        ratatui::init(),
        Tracker::new(combatants),
    );

    tracker.run();
}
