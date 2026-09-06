use color_eyre::eyre::{Result, eyre};
use crossterm::event::{self, Event, KeyCode};
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
// LaTeX Math zu Unicode Konvertierung
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
                let converted = convert_latex_symbols(&formula);
                if is_block {
                    out.push_str(&format!("\n   {}\n", converted.trim()));
                } else {
                    out.push_str(&converted);
                }
            } else {
                // Nicht geschlossenes '$': Roh ausgeben
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

fn convert_latex_symbols(latex: &str) -> String {
    let mut s = latex.to_string();

    // Mengenbereiche & Blackboard
    s = s.replace(r"\mathbb{R}", "ℝ");
    s = s.replace(r"\mathbb{N}", "ℕ");
    s = s.replace(r"\mathbb{Z}", "ℤ");
    s = s.replace(r"\mathbb{Q}", "ℚ");
    s = s.replace(r"\mathbb{C}", "ℂ");

    // Beweis- und Logikoperatoren
    s = s.replace(r"\iff", "⟺");
    s = s.replace(r"\implies", "⟹");
    s = s.replace(r"\to", "→");
    s = s.replace(r"\forall", "∀");
    s = s.replace(r"\exists", "∃");
    s = s.replace(r"\nexists", "∄");
    s = s.replace(r"\land", "∧");
    s = s.replace(r"\lor", "∨");
    s = s.replace(r"\neg", "¬");
    s = s.replace(r"\qed", "∎");

    // Mengenlehre & Relationen
    s = s.replace(r"\in", "∈");
    s = s.replace(r"\notin", "∉");
    s = s.replace(r"\subset", "⊂");
    s = s.replace(r"\subseteq", "⊆");
    s = s.replace(r"\cup", "∪");
    s = s.replace(r"\cap", "∩");
    s = s.replace(r"\setminus", "∖");
    s = s.replace(r"\empty", "∅");
    s = s.replace(r"\emptyset", "∅");

    // Vergleich & Arithmetik
    s = s.replace(r"\le", "≤");
    s = s.replace(r"\leq", "≤");
    s = s.replace(r"\ge", "≥");
    s = s.replace(r"\geq", "≥");
    s = s.replace(r"\neq", "≠");
    s = s.replace(r"\approx", "≈");
    s = s.replace(r"\equiv", "≡");
    s = s.replace(r"\cdot", "·");
    s = s.replace(r"\times", "×");
    s = s.replace(r"\pm", "±");
    s = s.replace(r"\infty", "∞");

    // Griechische Buchstaben (häufig in Beweisen)
    s = s.replace(r"\alpha", "α");
    s = s.replace(r"\beta", "β");
    s = s.replace(r"\gamma", "γ");
    s = s.replace(r"\Gamma", "Γ");
    s = s.replace(r"\delta", "δ");
    s = s.replace(r"\Delta", "Δ");
    s = s.replace(r"\epsilon", "ε");
    s = s.replace(r"\varepsilon", "ε");
    s = s.replace(r"\zeta", "ζ");
    s = s.replace(r"\eta", "η");
    s = s.replace(r"\theta", "θ");
    s = s.replace(r"\lambda", "λ");
    s = s.replace(r"\mu", "μ");
    s = s.replace(r"\pi", "π");
    s = s.replace(r"\sigma", "σ");
    s = s.replace(r"\Sigma", "Σ");
    s = s.replace(r"\tau", "τ");
    s = s.replace(r"\phi", "φ");
    s = s.replace(r"\omega", "ω");
    s = s.replace(r"\Omega", "Ω");

    // Wurzeln & Brüche
    s = replace_unary_cmd(&s, r"\sqrt", "√");
    s = replace_frac(&s);
    s = replace_unary_cmd(&s, r"\text", "");

    // Exponenten und Indizes
    s = convert_sub_and_superscripts(&s);

    s
}

fn replace_unary_cmd(input: &str, cmd: &str, prefix: &str) -> String {
    let mut res = input.to_string();
    while let Some(start) = res.find(cmd) {
        let after = &res[start + cmd.len()..];
        if let Some(open) = after.find('{') {
            if after[..open].trim().is_empty() {
                if let Some(close) = find_matching_brace(&after[open..]) {
                    let inner = &after[open + 1..open + close];
                    let whole = &res[start..start + cmd.len() + open + close + 1];
                    let replacement = format!("{}{}", prefix, inner);
                    res = res.replacen(whole, &replacement, 1);
                    continue;
                }
            }
        }
        break;
    }
    res
}

fn replace_frac(input: &str) -> String {
    let mut res = input.to_string();
    while let Some(start) = res.find(r"\frac") {
        let after = &res[start + 5..];
        if let Some(open1) = after.find('{') {
            if let Some(close1) = find_matching_brace(&after[open1..]) {
                let num = &after[open1 + 1..open1 + close1];
                let rem = &after[open1 + close1 + 1..];
                if let Some(open2) = rem.find('{') {
                    if let Some(close2) = find_matching_brace(&rem[open2..]) {
                        let den = &rem[open2 + 1..open2 + close2];
                        let total_len = 5 + open1 + close1 + 1 + open2 + close2 + 1;
                        let whole = &res[start..start + total_len];
                        let replacement = format!("({} / {})", num.trim(), den.trim());
                        res = res.replacen(whole, &replacement, 1);
                        continue;
                    }
                }
            }
        }
        break;
    }
    res
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (idx, ch) in s.char_indices() {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

fn convert_sub_and_superscripts(input: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c == '^' || c == '_' {
            let is_super = c == '^';
            i += 1;
            if i >= chars.len() {
                out.push(c);
                break;
            }

            if chars[i] == '{' {
                i += 1;
                while i < chars.len() && chars[i] != '}' {
                    out.push(map_script_char(chars[i], is_super));
                    i += 1;
                }
                if i < chars.len() && chars[i] == '}' {
                    i += 1;
                }
            } else {
                out.push(map_script_char(chars[i], is_super));
                i += 1;
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

fn map_script_char(c: char, is_super: bool) -> char {
    if is_super {
        match c {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            '+' => '⁺',
            '-' => '⁻',
            '=' => '⁼',
            '(' => '⁽',
            ')' => '⁾',
            'n' => 'ⁿ',
            'i' => 'ⁱ',
            'x' => 'ˣ',
            _ => c,
        }
    } else {
        match c {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            '+' => '₊',
            '-' => '₋',
            '=' => '₌',
            '(' => '₍',
            ')' => '₎',
            'a' => 'ₐ',
            'e' => 'ₑ',
            'o' => 'ₒ',
            'x' => 'ₓ',
            'i' => 'ᵢ',
            'j' => 'ⱼ',
            'k' => 'ₖ',
            'l' => 'ₗ',
            'm' => 'ₘ',
            'n' => 'ₙ',
            'p' => 'ₚ',
            's' => 'ₛ',
            't' => 'ₜ',
            _ => c,
        }
    }
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
                "FRAGE:\n{}\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nANTWORT:\n{}",
                question_rendered, answer_rendered
            )
        } else {
            format!(
                "FRAGE:\n{}\n\n\n[Leertaste] drücken, um Antwort anzuzeigen",
                question_rendered
            )
        };

        let card_block = Paragraph::new(content)
            .block(
                Block::bordered()
                    .title(title)
                    .border_type(BorderType::Double)
                    // Innenabstand: 2 Zeilen oben/unten, 3 Zeichen links/rechts
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
