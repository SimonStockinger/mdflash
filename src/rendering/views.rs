use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, BorderType, List, ListItem, Padding, Paragraph, Widget, Wrap};

use crate::parse_latex::*;
use crate::parse_markdown::parse_markdown_to_text;
use crate::{AppMode, AppState, CardKind};

pub fn render(frame: &mut Frame, app_state: &AppState) {
    match app_state.mode {
        AppMode::Directory => render_directory_view(frame, app_state),
        AppMode::DirectFile => render_card_view(frame, app_state),
    }
}

pub fn render_directory_view(frame: &mut Frame, app_state: &AppState) {
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

pub fn render_card_view(frame: &mut Frame, app_state: &AppState) {
    let deck = match &app_state.deck {
        Some(d) => d,
        None => return,
    };

    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Fill(1),   // Flashcard
        Constraint::Length(2), // Help & Navigation
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

        let raw_text = if app_state.show_answer {
            let answer_rendered = render_math_in_text(&card.answer);
            format!(
                "{}\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n{}",
                question_rendered, answer_rendered
            )
        } else {
            format!("{}\n", question_rendered)
        };

        let styled_text = parse_markdown_to_text(&raw_text);

        let card_block = Paragraph::new(styled_text)
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
            .scroll((app_state.scroll_offset, 0))
            .wrap(Wrap { trim: false });

        card_block.render(chunks[1], frame.buffer_mut());
    } else {
        let empty_block = Paragraph::new("No Flashcards found.")
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .padding(Padding::uniform(1)),
            )
            .style(Style::default().fg(Color::Red));
        empty_block.render(chunks[1], frame.buffer_mut());
    }

    let help_text = Paragraph::new(
        "[Space] Show Answer  |  [↑ / ↓] Scroll  |  [→ / N] Next  |  [← / B] Back  |  [Esc / Q] Quit",
    )
    .style(Style::default().fg(Color::DarkGray));
    help_text.render(chunks[2], frame.buffer_mut());
}
