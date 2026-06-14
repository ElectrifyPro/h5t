use canvas::Canvas;
use crate::{
    input::{AfterKey as AfterKeyInner, Charset, GetInput},
    selectable::Selectable,
    theme::THEME,
    view::LABELS,
    widgets::{ability_scores::score_to_color, popup::Select},
};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::{
    ability::Score,
    monster::MONSTERS,
    Ability,
    Combatant,
    CombatantKind,
    score_to_modifier,
};
use ratatui::{layout::Flex, prelude::*, widgets::*};
use std::collections::HashMap;
use super::AfterKey;

#[derive(Clone, Debug)]
enum Step {
    /// Whether to add a player-controlled character or pre-existing monster.
    ChooseKind(Option<CombatantKindLabel>),

    /// Adding a pre-existing monster.
    AddMonster {
        /// Index of the selected monster in the table **filtered to the current search query**.
        selected: usize,
    },
}

/// Helper enum to choose a combatant kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CombatantKindLabel {
    Player,
    Monster,
}

impl Selectable for CombatantKindLabel {
    const N: usize = 2;

    fn variants() -> impl Iterator<Item = Self> {
        [
            CombatantKindLabel::Player,
            CombatantKindLabel::Monster,
        ].into_iter()
    }
}

impl std::fmt::Display for CombatantKindLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Creates an iterator of [`Text`]s for displaying a monster's ability modifiers in one line.
fn modifier_line(scores: Ability<Score>) -> impl Iterator<Item = Text<'static>> {
    let make_span = |score: Score| {
        let color = score_to_color(score);
        let modifier = score_to_modifier(score);
        Text::styled(format!("{:+}", modifier), color)
    };

    [
        make_span(scores.strength),
        make_span(scores.dexterity),
        make_span(scores.constitution),
        make_span(scores.intelligence),
        make_span(scores.wisdom),
        make_span(scores.charisma),
    ]
        .into_iter()
}

/// State for adding a combatant to the combat.
#[derive(Clone, Debug)]
pub struct AddCombatant {
    /// The current step of this state.
    step: Step,

    /// Helper for the search bar in the monster lookup view.
    search: GetInput<String>,
}

impl AddCombatant {
    /// Create an [`AddCombatant`] state.
    pub fn new() -> Self {
        Self {
            step: Step::ChooseKind(None),
            search: GetInput::new("Search monsters", 20, Charset::All),
        }
    }

    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        // TODO: copied from select widget
        let area = frame.area();

        let [choose_kind, editor] = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
            .flex(Flex::Center)
            .areas(area);
        let [search, editor_content] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
            .areas(editor);

        match &self.step {
            Step::ChooseKind(kind) => {
                frame.render_widget(Select::new(
                    "Select combatant to add",
                    kind.as_ref(),
                    true,
                ), choose_kind);
            },
            Step::AddMonster { selected } => {
                frame.render_widget(Select::with_selected(
                    "Select combatant to add",
                    &CombatantKindLabel::Monster,
                    false,
                ), choose_kind);

                self.search.draw(frame, search);

                // clear the area for the monster search table
                Clear.render(editor_content, frame.buffer_mut());
                Widget::render(
                    Canvas::default()
                        .background_color(THEME.background.into())
                        .paint(|_| ()),
                    editor_content,
                    frame.buffer_mut(),
                );

                let widget = Table::new(
                    MONSTERS.iter()
                        .filter(|m| m.name.to_lowercase().contains(self.search.as_str()))
                        .enumerate() // `index` refers to monsters **filtered** in
                        .map(|(i, m)| {
                            let is_selected = i == *selected;

                            let monster_name = Text::raw(&m.name);
                            let challenge = Text::from(m.challenge_rating.to_string());
                            let xp = Text::from(m.xp.to_string());
                            let hp = Text::from(m.hit_points.to_string())
                                .alignment(Alignment::Right);

                            let mut style = Style::default().fg(THEME.foreground.into());
                            if is_selected {
                                style = style
                                    .bold()
                                    .bg(THEME.select.into());
                            }

                            Row::new(
                                [monster_name, challenge, xp].into_iter()
                                    .chain(modifier_line(m.scores))
                                    .chain(Some(hp))
                            )
                                .style(style)
                        }),
                    [
                        Constraint::Fill(1),   // monster name
                        Constraint::Length(9), // challenge rating
                        Constraint::Length(6), // xp
                        Constraint::Length(3), // str modifier
                        Constraint::Length(3), // dex modifier
                        Constraint::Length(3), // con modifier
                        Constraint::Length(3), // int modifier
                        Constraint::Length(3), // wis modifier
                        Constraint::Length(3), // cha modifier
                        Constraint::Length(5), // base hp
                    ],
                )
                    .header(Row::new(vec![
                        Text::from("Monster"),
                        Text::from("Challenge"),
                        Text::from("XP"),
                        Text::from("STR"),
                        Text::from("DEX"),
                        Text::from("CON"),
                        Text::from("INT"),
                        Text::from("WIS"),
                        Text::from("CHA"),
                        Text::from("HP"),
                    ]).bold())
                    .fg(THEME.foreground)
                    .block(Block::bordered()
                        .border_type(BorderType::Rounded)
                        .border_style(THEME.foreground)
                        .title("↑ / ↓ = select and scroll")
                        .padding(Padding::symmetric(1, 0)));

                Widget::render(widget, editor_content, frame.buffer_mut());
            },
        }
    }

    /// Handle a key event and apply any needed changes to the combatant list.
    pub fn handle_key(&mut self, key: KeyEvent, combatants: &mut Vec<Combatant>) -> AfterKey {
        match &mut self.step {
            Step::ChooseKind(kind) => match key.code {
                KeyCode::Esc => AfterKey::Exit,
                KeyCode::Enter => {
                    // two ways to reach add player / monster step
                    match kind {
                        Some(CombatantKindLabel::Player) => (), // TODO
                        Some(CombatantKindLabel::Monster) => self.step = Step::AddMonster {
                            selected: 0,
                        },
                        _ => (),
                    }
                    AfterKey::Stay
                },
                KeyCode::Char(label) => {
                    let label_to_option = LABELS
                        .chars()
                        .zip(CombatantKindLabel::variants())
                        .collect::<HashMap<_, _>>();

                    if let Some(&option) = label_to_option.get(&label) {
                        if *kind == Some(option) {
                            *kind = None;
                        } else {
                            *kind = Some(option);
                            if matches!(option, CombatantKindLabel::Monster) {
                                self.step = Step::AddMonster {
                                    selected: 0,
                                };
                            }
                        }
                    }

                    AfterKey::Stay
                },
                _ => AfterKey::Stay,
            },
            Step::AddMonster { selected } => match self.search.handle_key(key) {
                AfterKeyInner::Handled => {
                    *selected = 0;
                    AfterKey::Stay
                },
                AfterKeyInner::Submit(_) => {
                    // add monster, but leave window open so more can be added
                    let maybe_monster = MONSTERS.iter()
                        .filter(|m| m.name.to_lowercase().contains(self.search.as_str()))
                        .nth(*selected);
                    if let Some(monster) = maybe_monster {
                        combatants.push(CombatantKind::Monster(monster.clone()).into());
                    }
                    AfterKey::Stay
                },
                AfterKeyInner::Cancel => {
                    self.step = Step::ChooseKind(Some(CombatantKindLabel::Monster));
                    AfterKey::Stay
                },
                AfterKeyInner::Forward(event) => {
                    match event.code {
                        KeyCode::Up => {
                            *selected = selected.saturating_sub(1);
                        },
                        KeyCode::Down => {
                            *selected += 1;
                        },
                        _ => (),
                    }
                    AfterKey::Stay
                },
            },
        }
    }
}
