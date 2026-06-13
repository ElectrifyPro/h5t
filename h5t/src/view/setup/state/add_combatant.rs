use crate::{
    input::{AfterKey as AfterKeyInner, Charset, GetInput},
    selectable::Selectable,
    theme::THEME,
    view::LABELS,
    widgets::popup::Select,
};
use crossterm::event::{KeyCode, KeyEvent};
use h5t_core::{ability::Score, monster::MONSTERS, Ability, Combatant, score_to_modifier};
use ratatui::{layout::Flex, prelude::*, widgets::*};
use std::collections::HashMap;
use super::AfterKey;

#[derive(Clone, Debug)]
enum Step {
    /// Whether to add a player-controlled character or pre-existing monster.
    ChooseKind(Option<CombatantKindLabel>),

    /// Adding a pre-existing monster.
    AddMonster,
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

/// Creates an iterator of [`Span`]s for displaying a monster's ability modifiers in one line.
fn modifier_line(scores: Ability<Score>) -> impl Iterator<Item = Text<'static>> {
    // TODO: copied from ability_scores.rs
    // more green for high scores, more red for low scores
    // 0: (255, 0, 0)
    // 10: (255, 255, 255)
    // 20: (0, 255, 0)
    fn score_to_color(score: i32) -> Color {
        Color::Rgb(
            (510.0 - 255.0 / 10.0 * score as f32).min(255.0) as u8,
            (255.0 / 10.0 * score as f32).min(255.0) as u8,
            (255.0 - (255.0 / 10.0 * score as f32 - 255.0).abs()).max(0.0) as u8,
        )
    }

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
            Step::AddMonster => {
                frame.render_widget(Select::with_selected(
                    "Select combatant to add",
                    &CombatantKindLabel::Monster,
                    false,
                ), choose_kind);

                self.search.draw(frame, search);

                let widget = Table::new(
                    MONSTERS.iter()
                        .filter(|m| m.name.to_lowercase().contains(self.search.as_str()))
                        .map(|m| {
                            let monster_name = Text::raw(&m.name);
                            let challenge = Text::from(m.challenge_rating.to_string());
                            let xp = Text::from(m.xp.to_string());
                            let hp = Text::from(m.hit_points.to_string())
                                .alignment(Alignment::Right);
                            Row::new(
                                [monster_name, challenge, xp].into_iter()
                                    .chain(modifier_line(m.scores))
                                    .chain(Some(hp))
                            )
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
    pub fn handle_key(&mut self, key: KeyEvent, _combatants: &mut Vec<Combatant>) -> AfterKey {
        match &mut self.step {
            Step::ChooseKind(kind) => match key.code {
                KeyCode::Esc => AfterKey::Exit,
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
                                self.step = Step::AddMonster;
                            }
                        }
                    }

                    AfterKey::Stay
                },
                _ => AfterKey::Stay,
            },
            Step::AddMonster => match self.search.handle_key(key) {
                AfterKeyInner::Handled => {
                    // TODO:
                    AfterKey::Stay
                },
                AfterKeyInner::Submit(_) => {
                    // TODO:
                    AfterKey::Exit
                },
                AfterKeyInner::Cancel => {
                    self.step = Step::ChooseKind(Some(CombatantKindLabel::Monster));
                    AfterKey::Stay
                },
                AfterKeyInner::Forward(_event) => {
                    // TODO:
                    AfterKey::Stay
                },
            },
        }
    }
}
