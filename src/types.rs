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
