mod input;
mod selectable;
mod theme;
mod view;
mod widgets;

use h5t_core::{monster::MONSTERS, CombatantKind, Spell, Tracker};
use view::battle::Battle;
use view::setup::Setup;

fn main() {
    // NOTE: spell JSON data provided courtesy of https://www.dnd5eapi.co/
    let file = std::fs::File::open("data/spells.json").unwrap();
    let spells = serde_json::from_reader::<_, Vec<Spell>>(file).unwrap();

    let mut combatants = MONSTERS
        .iter()
        .cloned()
        .map(|m| {
            let mut combatant = Combatant::from(m);
            combatant.group = Some(String::from("Monsters"));
            combatant
        })
        .collect::<Vec<_>>();

    use h5t_core::{ability::{Proficiencies, Skill}, monster::ArmorClass, resource::ResourcePool, speed::Speed, Ability, Character, Combatant};
    use std::collections::HashMap;
    let pcs = vec![
        Combatant {
            initiative: None,
            group: Some(String::from("Players")),
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
            health: 32.try_into().unwrap(),
            resource_pool: ResourcePool::default(),
        },
    ];
    combatants.splice(0..0, pcs);

    let mut terminal = ratatui::init();

    let mut setup = Setup::new();
    setup.run(&mut terminal);

    let inner = setup.inner;

    // TODO: will crash if there are no combatants, please improve
    let mut tracker = Battle::new(Tracker::new(inner.combatants), inner.groups);
    tracker.run(&mut terminal);

    ratatui::restore();
}
