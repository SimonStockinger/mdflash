use std::slice::from_mut;

use color_eyre::eyre::{Ok, Result};
use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Stylize},
    widgets::{Block, List, ListItem, Widget},
};

#[derive(Debug, Default)]
struct AppState {
    path: &'static str,
    data: Data,
}

#[derive(Debug, Default)]
struct Data {
    decks: Vec<CardDeck>,
}

#[derive(Debug, Default)]
struct CardDeck {
    title: &'static str,
    date: &'static str,
    index: u32,
    total_number_of_cards: u32,
    cards_left: u32,
    flashcards: Vec<FlashCard>,
}

#[derive(Debug, Default)]
struct FlashCard {
    title: &'static str,
    question: &'static str,
    answer: &'static str,
    finished: bool,
}

fn main() -> Result<()> {
    let mut state = AppState::default();
    init(&mut state);

    color_eyre::install();
    let terminal = ratatui::init();

    let res = run(terminal, &mut state);
    ratatui::restore();
    res
}

fn init(app_state: &mut AppState) -> Result<()> {
    let card = FlashCard {
        title: "TestCard",
        question: "Why?",
        answer: "That's why!",
        finished: false,
    };
    let cards = vec![card];
    app_state.data.decks.push(CardDeck {
        title: "Test",
        date: "2026-09-03",
        index: 0,
        total_number_of_cards: 1,
        cards_left: 1,
        flashcards: cards,
    });
    Ok(())
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        // Rendering
        terminal.draw(|f| render(f, app_state));
        // Input handling
        if let Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app_state: &AppState) {
    let [border_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());
    let [inner_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(border_area);
    Block::bordered()
        .border_type(ratatui::widgets::BorderType::Rounded)
        .fg(Color::DarkGray)
        .render(border_area, frame.buffer_mut());

    List::new(
        app_state
            .data
            .decks
            .iter()
            .map(|x| ListItem::from(x.title.clone())),
    )
    .render(inner_area, frame.buffer_mut())
}
