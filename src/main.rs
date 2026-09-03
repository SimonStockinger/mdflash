use color_eyre::eyre::{Ok, Result};
use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Stylize},
    widgets::{Block, List, ListItem, Widget},
};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default)]
struct AppState {
    path: PathBuf,
    data: Data,
}

#[derive(Debug, Default)]
struct Data {
    dir_files: Vec<PathBuf>,
}

#[derive(Debug, Default)]
struct CardDeck {
    title: &'static str,
    path: &'static str,
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
    let mut state = init().unwrap();

    color_eyre::install();
    let terminal = ratatui::init();

    let res = run(terminal, &mut state);
    ratatui::restore();
    res
}

fn init() -> Result<AppState> {
    let path = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let path_buf = PathBuf::from(&path);

    let entries = fs::read_dir(&path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;

    let state = AppState {
        path: path_buf,
        data: Data { dir_files: entries },
    };

    Ok(state)
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

    let items = app_state
        .data
        .dir_files
        .iter()
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("md"))
        })
        .map(|path| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned());

            ListItem::new(name)
        });

    let list = List::new(items);

    list.render(inner_area, frame.buffer_mut())
}
