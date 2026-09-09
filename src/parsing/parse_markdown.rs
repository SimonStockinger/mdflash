use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use std::path::Path;

use crate::*;

#[derive(Default)]
pub struct ParsedMeta {
    deck_name: Option<String>,
    tags: Vec<String>,
}

pub fn parse_markdown_to_text(input: &str) -> Text<'static> {
    let mut lines = Vec::new();
    for line in input.lines() {
        lines.push(parse_markdown_line(line));
    }
    Text::from(lines)
}

pub fn parse_markdown_line(line: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(&c) = chars.peek() {
        // Inline Code `...`
        if c == '`' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            chars.next();
            let mut code = String::new();
            while let Some(&inner) = chars.peek() {
                chars.next();
                if inner == '`' {
                    break;
                }
                code.push(inner);
            }
            spans.push(Span::styled(
                code,
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Rgb(45, 45, 45)),
            ));
            continue;
        }

        // Durchgestrichen ~~...~~
        if c == '~' {
            let mut clone_iter = chars.clone();
            clone_iter.next();
            if clone_iter.peek() == Some(&'~') {
                if !current.is_empty() {
                    spans.push(Span::raw(current.clone()));
                    current.clear();
                }
                chars.next();
                chars.next();
                let mut inner = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '~' && chars.peek() == Some(&'~') {
                        chars.next();
                        break;
                    }
                    inner.push(ch);
                }
                spans.push(Span::styled(
                    inner,
                    Style::default().add_modifier(Modifier::CROSSED_OUT),
                ));
                continue;
            }
        }

        // Fett & Kursiv (***, **, *, ___, __, _)
        if c == '*' || c == '_' {
            let marker = c;
            let count = count_markers(&mut chars.clone(), marker);

            if count > 0 {
                if !current.is_empty() {
                    spans.push(Span::raw(current.clone()));
                    current.clear();
                }

                for _ in 0..count {
                    chars.next();
                }

                let mut inner = String::new();
                let mut closed = false;

                while let Some(ch) = chars.next() {
                    if ch == marker {
                        let mut match_count = 1;
                        while match_count < count && chars.peek() == Some(&marker) {
                            chars.next();
                            match_count += 1;
                        }
                        if match_count == count {
                            closed = true;
                            break;
                        } else {
                            for _ in 0..match_count {
                                inner.push(marker);
                            }
                        }
                    } else {
                        inner.push(ch);
                    }
                }

                if closed {
                    let mut style = Style::default();
                    match count {
                        1 => style = style.add_modifier(Modifier::ITALIC),
                        2 => style = style.add_modifier(Modifier::BOLD),
                        _ => style = style.add_modifier(Modifier::BOLD | Modifier::ITALIC),
                    }
                    spans.push(Span::styled(inner, style));
                } else {
                    for _ in 0..count {
                        current.push(marker);
                    }
                    current.push_str(&inner);
                }
                continue;
            }
        }

        current.push(c);
        chars.next();
    }

    if !current.is_empty() {
        spans.push(Span::raw(current));
    }

    Line::from(spans)
}

pub fn count_markers(chars: &mut std::iter::Peekable<std::str::Chars>, marker: char) -> usize {
    let mut count = 0;
    while let Some(&c) = chars.peek() {
        if c == marker {
            count += 1;
            chars.next();
            if count == 3 {
                break;
            }
        } else {
            break;
        }
    }
    count
}

pub fn strip_and_parse_frontmatter(content: &str) -> (ParsedMeta, &str) {
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

pub fn parse_markdown_deck(path: &Path, content: &str) -> CardDeck {
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

pub fn parse_cards(body: &str) -> Vec<FlashCard> {
    let mut cards = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Abschnitte oder Blockquotes (> ...) überspringen
        if line.starts_with('#') || line.starts_with('>') || line.is_empty() {
            i += 1;
            continue;
        }

        // Markdown-Tabellen ignorieren
        if line.starts_with('|') {
            i += 1;
            continue;
        }

        if is_multiline_separator(line) {
            i += 1;
            continue;
        }

        // Multi-Line Card (Frage gefolgt von einer Zeile mit "?")
        if i + 1 < lines.len() && is_multiline_separator(lines[i + 1].trim()) {
            let question = line.to_string();
            let mut answer_lines = Vec::new();
            i += 2;

            while i < lines.len() {
                let curr = lines[i].trim();
                // Abbruchbedingung: Neuer Header, neue Singleline/Reversed/Cloze-Karte oder nächster Trenner
                if curr.starts_with('#')
                    || curr.starts_with('>')
                    || curr.starts_with('|')
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

pub fn is_multiline_separator(line: &str) -> bool {
    line == "?"
}

pub fn is_cloze_line(line: &str) -> bool {
    if let Some(first) = line.find("==") {
        line[first + 2..].contains("==")
    } else {
        false
    }
}

pub fn make_cloze_card(line: &str) -> Option<FlashCard> {
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
