use color_eyre::eyre::{Result, eyre};
use crossterm::event::{self, Event, KeyCode};
use latex2unicode::latex2unicode;
use ratatui::widgets::{Block, BorderType, List, ListItem, Padding, Paragraph, Widget, Wrap};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum AppMode {
    Directory,
    DirectFile,
}

#[derive(Debug)]
struct AppState {
    mode: AppMode,
    path: PathBuf,
    data: Data,
    deck: Option<CardDeck>,
    current_card_index: usize,
    show_answer: bool,
}

#[derive(Debug, Default)]
struct Data {
    dir_files: Vec<PathBuf>,
}

#[derive(Debug, Default, Clone)]
struct CardDeck {
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
        })
    }
}

// -----------------------------------------------------------------------------
// LaTeX Math zu Unicode Konvertierung via latex2unicode Crate
// -----------------------------------------------------------------------------

fn render_math_in_text(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            let is_block = chars.peek() == Some(&'$');
            if is_block {
                chars.next(); // zweites '$' verbrauchen
            }

            let mut formula = String::new();
            let mut closed = false;

            while let Some(c) = chars.next() {
                if c == '$' {
                    if is_block {
                        if chars.peek() == Some(&'$') {
                            chars.next();
                            closed = true;
                            break;
                        } else {
                            formula.push(c);
                        }
                    } else {
                        closed = true;
                        break;
                    }
                } else {
                    formula.push(c);
                }
            }

            if closed {
                let converted = latex2unicode(&formula);
                if is_block {
                    out.push_str(&format!("\n    {}\n", converted.trim()));
                } else {
                    out.push_str(&converted);
                }
            } else {
                if is_block {
                    out.push_str("$$");
                } else {
                    out.push('$');
                }
                out.push_str(&formula);
            }
        } else {
            out.push(ch);
        }
    }

    out
}

// -----------------------------------------------------------------------------
// Markdown Deck & Card Parsing
// -----------------------------------------------------------------------------

#[derive(Default)]
struct ParsedMeta {
    deck_name: Option<String>,
    tags: Vec<String>,
}

fn strip_and_parse_frontmatter(content: &str) -> (ParsedMeta, &str) {
    let mut meta = ParsedMeta::default();
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return (meta, content);
    }

    if let Some(rest) = trimmed.strip_prefix("---") {
        if let Some((yaml_block, body)) = rest.split_once("\n---") {
            let mut in_tags_list = false;

            for raw_line in yaml_block.lines() {
                let line = raw_line.trim();

                if line.starts_with("deck:") {
                    in_tags_list = false;
                    meta.deck_name = line
                        .strip_prefix("deck:")
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string());
                } else if line.starts_with("tags:") {
                    in_tags_list = true;
                } else if in_tags_list && line.starts_with('-') {
                    if let Some(tag) = line.strip_prefix('-') {
                        meta.tags.push(tag.trim().to_string());
                    }
                } else if !line.starts_with('#') && !line.is_empty() {
                    in_tags_list = false;
                }
            }

            return (meta, body.trim_start());
        }
    }

    (meta, content)
}

fn parse_markdown_deck(path: &Path, content: &str) -> CardDeck {
    let (frontmatter, body) = strip_and_parse_frontmatter(content);

    let default_title = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Deck".to_string());

    let title = frontmatter
        .deck_name
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(default_title);

    let flashcards = parse_cards(body);

    CardDeck {
        title,
        tags: frontmatter.tags,
        path: path.to_path_buf(),
        flashcards,
    }
}

fn parse_cards(body: &str) -> Vec<FlashCard> {
    let mut cards = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with('#') || line.is_empty() {
            i += 1;
            continue;
        }

        if is_multiline_separator(line) {
            i += 1;
            continue;
        }

        // Multi-Line Card
        if i + 1 < lines.len() && is_multiline_separator(lines[i + 1].trim()) {
            let question = line.to_string();
            let mut answer_lines = Vec::new();
            i += 2;

            while i < lines.len() {
                let curr = lines[i].trim();
                if curr.starts_with('#')
                    || curr.contains("::")
                    || (i + 1 < lines.len() && is_multiline_separator(lines[i + 1].trim()))
                    || is_cloze_line(curr)
                {
                    break;
                }
                answer_lines.push(lines[i]);
                i += 1;
            }

            cards.push(FlashCard {
                question,
                answer: answer_lines.join("\n").trim().to_string(),
                kind: CardKind::MultiLine,
            });
            continue;
        }

        // Reversed Card (:::)
        if let Some((q, a)) = line.split_once(":::") {
            let q = q.trim().to_string();
            let a = a.trim().to_string();

            cards.push(FlashCard {
                question: q.clone(),
                answer: a.clone(),
                kind: CardKind::Reversed,
            });
            cards.push(FlashCard {
                question: a,
                answer: q,
                kind: CardKind::Reversed,
            });

            i += 1;
            continue;
        }

        // Single-Line Card (::)
        if let Some((q, a)) = line.split_once("::") {
            cards.push(FlashCard {
                question: q.trim().to_string(),
                answer: a.trim().to_string(),
                kind: CardKind::SingleLine,
            });
            i += 1;
            continue;
        }

        // Cloze Deletion (==...==)
        if is_cloze_line(line) {
            if let Some(card) = make_cloze_card(line) {
                cards.push(card);
            }
            i += 1;
            continue;
        }

        i += 1;
    }

    cards
}

fn is_multiline_separator(line: &str) -> bool {
    line == "?"
}

fn is_cloze_line(line: &str) -> bool {
    if let Some(first) = line.find("==") {
        line[first + 2..].contains("==")
    } else {
        false
    }
}

fn make_cloze_card(line: &str) -> Option<FlashCard> {
    let mut parts = line.split("==");
    let mut question = String::new();
    let mut answers = Vec::new();
    let mut is_highlight = false;

    while let Some(part) = parts.next() {
        if is_highlight {
            answers.push(part.trim().to_string());
            question.push_str("[...]");
        } else {
            question.push_str(part);
        }
        is_highlight = !is_highlight;
    }

    if answers.is_empty() {
        return None;
    }

    Some(FlashCard {
        question: question.trim().to_string(),
        answer: format!(
            "Lösung: {}\n\nVollständiger Satz:\n{}",
            answers.join(", "),
            line
        ),
        kind: CardKind::Cloze,
    })
}

// -----------------------------------------------------------------------------
// UI & Event Loop
// -----------------------------------------------------------------------------

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app_state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => break,
                KeyCode::Char(' ') => {
                    app_state.show_answer = !app_state.show_answer;
                }
                KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('n') => {
                    if let Some(deck) = &app_state.deck {
                        if !deck.flashcards.is_empty()
                            && app_state.current_card_index + 1 < deck.flashcards.len()
                        {
                            app_state.current_card_index += 1;
                            app_state.show_answer = false;
                        }
                    }
                }
                KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('p') => {
                    if app_state.current_card_index > 0 {
                        app_state.current_card_index -= 1;
                        app_state.show_answer = false;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app_state: &AppState) {
    match app_state.mode {
        AppMode::Directory => render_directory_view(frame, app_state),
        AppMode::DirectFile => render_card_view(frame, app_state),
    }
}

fn render_directory_view(frame: &mut Frame, app_state: &AppState) {
    let [border_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());
    let [inner_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(border_area);

    Block::bordered()
        .border_type(BorderType::Rounded)
        .title(" Directory Browser ")
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
    list.render(inner_area, frame.buffer_mut());
}

fn render_card_view(frame: &mut Frame, app_state: &AppState) {
    let deck = match &app_state.deck {
        Some(d) => d,
        None => return,
    };

    let chunks = Layout::vertical([
        Constraint::Length(3), // Titel / Metadaten
        Constraint::Fill(1),   // Kartenbereich
        Constraint::Length(2), // Hilfe-Footer
    ])
    .margin(1)
    .split(frame.area());

    let progress = if !deck.flashcards.is_empty() {
        format!(
            "{}/{}",
            app_state.current_card_index + 1,
            deck.flashcards.len()
        )
    } else {
        "0/0".to_string()
    };

    let title_block = Paragraph::new(format!("Deck: {} | Karte: {}", deck.title, progress))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1)),
        )
        .style(Style::default().fg(Color::Cyan));
    title_block.render(chunks[0], frame.buffer_mut());

    if let Some(card) = deck.flashcards.get(app_state.current_card_index) {
        let kind_badge = match card.kind {
            CardKind::SingleLine => " [Single-Line] ",
            CardKind::Reversed => " [Reversed ⇄] ",
            CardKind::MultiLine => " [Multi-Line] ",
            CardKind::Cloze => " [Cloze / Lückentext] ",
        };

        let title = if app_state.show_answer {
            format!(" Karte (Aufgedeckt){} ", kind_badge)
        } else {
            format!(" Karte{} ", kind_badge)
        };

        let question_rendered = render_math_in_text(&card.question);

        let content = if app_state.show_answer {
            let answer_rendered = render_math_in_text(&card.answer);
            format!(
                "{}\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n{}",
                question_rendered, answer_rendered
            )
        } else {
            format!("{}\n", question_rendered)
        };

        let card_block = Paragraph::new(content)
            .block(
                Block::bordered()
                    .title(title)
                    .border_type(BorderType::Double)
                    .padding(Padding::symmetric(3, 2))
                    .fg(if app_state.show_answer {
                        Color::Green
                    } else {
                        Color::Yellow
                    }),
            )
            .wrap(Wrap { trim: false });

        card_block.render(chunks[1], frame.buffer_mut());
    } else {
        let empty_block = Paragraph::new("Keine Karten in dieser Datei gefunden.")
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .padding(Padding::uniform(1)),
            )
            .style(Style::default().fg(Color::Red));
        empty_block.render(chunks[1], frame.buffer_mut());
    }

    let help_text = Paragraph::new(
        "[Space] Antwort zeigen  |  [→ / N] Nächste  |  [← / P] Vorherige  |  [Esc / Q] Beenden",
    )
    .style(Style::default().fg(Color::DarkGray));
    help_text.render(chunks[2], frame.buffer_mut());
}
