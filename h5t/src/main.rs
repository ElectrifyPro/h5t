mod monster;
mod tracker;

use crossterm::event::{read, Event, KeyCode};
use h5t_core::{CombatantKind, Monster, Tracker};
use ratatui::prelude::*;

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
            let layout = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]).split(frame.area());

            // print tracker
            frame.render_widget(tracker::TrackerWidget::new(&tracker), layout[0]);

            // print a nice card
            let combatant = tracker.current_combatant();
            let CombatantKind::Monster(monster) = &combatant.kind;
            frame.render_widget(monster::MonsterCard::new(monster), layout[1]);
        }).unwrap();
        if let Ok(Event::Key(key)) = read() {
            match key.code {
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

    ratatui::restore();
}
