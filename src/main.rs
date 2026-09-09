use color_eyre::eyre::{Result, eyre};
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;
use std::env;
use std::fs;
use std::path::PathBuf;

mod parsing;
mod rendering;

use crate::parsing::parse_markdown::parse_markdown_deck;
use crate::parsing::*;
use crate::rendering::views::render;

#[derive(Debug)]
pub enum AppMode {
    Directory,
    DirectFile,
}

#[derive(Debug)]
pub struct AppState {
    mode: AppMode,
    path: PathBuf,
    data: Data,
    deck: Option<CardDeck>,
    current_card_index: usize,
    show_answer: bool,
    scroll_offset: u16,
}

#[derive(Debug, Default)]
pub struct Data {
    dir_files: Vec<PathBuf>,
}

#[derive(Debug, Default, Clone)]
pub struct CardDeck {
    title: String,
    tags: Vec<String>,
    path: PathBuf,
    flashcards: Vec<FlashCard>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardKind {
    SingleLine,
    Reversed,
    MultiLine,
    Cloze,
}

#[derive(Debug, Clone)]
pub struct FlashCard {
    pub question: String,
    pub answer: String,
    pub kind: CardKind,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut state = init()?;
    let terminal = ratatui::init();

    let res = run(terminal, &mut state);
    ratatui::restore();
    res
}

fn init() -> Result<AppState> {
    let arg = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let path = PathBuf::from(&arg);

    if !path.exists() {
        return Err(eyre!("Pfad existiert nicht: {}", path.display()));
    }

    if path.is_file() {
        let content = fs::read_to_string(&path)?;
        let deck = parse_markdown_deck(&path, &content);

        Ok(AppState {
            mode: AppMode::DirectFile,
            path,
            data: Data::default(),
            deck: Some(deck),
            current_card_index: 0,
            show_answer: false,
            scroll_offset: 0,
        })
    } else {
        let entries = fs::read_dir(&path)?
            .map(|res| res.map(|e| e.path()))
            .collect::<std::io::Result<Vec<_>>>()?;

        Ok(AppState {
            mode: AppMode::Directory,
            path,
            data: Data { dir_files: entries },
            deck: None,
            current_card_index: 0,
            show_answer: false,
            scroll_offset: 0,
        })
    }
}

// UI & Event Loop
fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app_state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => break,
                KeyCode::Char(' ') => {
                    app_state.show_answer = !app_state.show_answer;
                    app_state.scroll_offset = 0;
                }
                KeyCode::Up => {
                    if app_state.scroll_offset > 0 {
                        app_state.scroll_offset -= 1;
                    }
                }
                KeyCode::Down => {
                    app_state.scroll_offset = app_state.scroll_offset.saturating_add(1);
                }
                KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('n') => {
                    if let Some(deck) = &app_state.deck {
                        if !deck.flashcards.is_empty()
                            && app_state.current_card_index + 1 < deck.flashcards.len()
                        {
                            app_state.current_card_index += 1;
                            app_state.show_answer = false;
                            app_state.scroll_offset = 0;
                        }
                    }
                }
                KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('b') => {
                    if app_state.current_card_index > 0 {
                        app_state.current_card_index -= 1;
                        app_state.show_answer = false;
                        app_state.scroll_offset = 0;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
