use std::path::Path;

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

#[derive(Debug, Default, Clone)]
pub struct CardDeck {
    pub title: String,
    pub tags: Vec<String>,
    pub flashcards: Vec<FlashCard>,
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
        flashcards,
    }
}

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

        // 3. Bi-directional / Reversed (:::)
        if let Some((q, a)) = line.split_once(":::") {
            let q = q.trim().to_string();
            let a = a.trim().to_string();

            // Forward
            cards.push(FlashCard {
                question: q.clone(),
                answer: a.clone(),
                kind: CardKind::Reversed,
            });
            // Reverse
            cards.push(FlashCard {
                question: a,
                answer: q,
                kind: CardKind::Reversed,
            });

            i += 1;
            continue;
        }

        // 4. Single-Line (::)
        if let Some((q, a)) = line.split_once("::") {
            cards.push(FlashCard {
                question: q.trim().to_string(),
                answer: a.trim().to_string(),
                kind: CardKind::SingleLine,
            });
            i += 1;
            continue;
        }

        // 5. Cloze Deletion (==Highlight==)
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

/// Ersetzt ==Antwort== durch [...] für die Frage und stellt die Lösung bereit
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
