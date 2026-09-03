# Guide: Markdown Flashcard Formats

A standardized format for Markdown flashcards allows you to easily convert and sync your notes with tools such as **Obsidian (Spaced Repetition Plugin)**, **Anki (via Markdown Import / AnkiConnect)**, **Logseq**, or custom study scripts.

---

## Standard Formats: Single-Line & Multi-Line (Obsidian / Anki Compatible)

This notation is the de facto standard used by tools like the _Obsidian Spaced Repetition Plugin_.

### A) Simple Question & Answer (Single-Line)

Use a double colon `::` as the separator.

```markdown
What is the primary goal of spaced repetition?::To optimize retention intervals through strategically timed reviews.
Which HTTP status code corresponds to "Not Found"?::404
```

### B) Bi-directional / Reversed Card

Use a triple colon `:::`. This automatically generates both a forward and a reverse card.

```markdown
Capital of France:::Paris
Photosynthesis:::The biological process of converting light energy into chemical energy
```

### C) Multi-Line Cards

Use a question mark `?` on its own line to separate the question from an extensive answer.

```markdown
What are the stages of the mitosis cycle?
?

1. Prophase
2. Metaphase
3. Anaphase
4. Telophase
   (Followed by cytokinesis)
```

---

## D) Cloze Deletions (Fill-in-the-Blank)

Ideal for learning definitions, formulas, or remembering key terms in context.

### Double Equals `==Highlight==` (Obsidian SR Standard)

```markdown
==Mitochondria== are the powerhouses of the cell.
Ohm's Law is defined as: ==V = I * R==.
```

---

## Metadata & Organization (Best Practice Template)

For organized and maintainable decks, add YAML frontmatter at the top of your Markdown file:

```markdown
---
deck: ComputerScience::Databases
tags:
  - flashcards
  - sql
  - databases
2026-09-03
author: Name
---

# Deck: Relational Databases

## Normalization

What is First Normal Form (1NF)?::A relation is in 1NF if every attribute domain contains only atomic values and there are no repeating groups.

What are the requirements for Second Normal Form (2NF)?
?

- The table must already satisfy **First Normal Form (1NF)**.
- Every non-key attribute must be **fully functionally dependent** on the entire primary key (no partial dependencies).

## SQL Basics

Which keyword removes duplicate rows from a query result set?::`SELECT DISTINCT`
```

---

## Separator Syntax Summary

| Card Type             | Syntax                                         | Typical Use Case                                     |
| :-------------------- | :--------------------------------------------- | :--------------------------------------------------- |
| **Single-Line**       | `Question::Answer`                             | Vocabulary, concise facts, dates                     |
| **Reversed**          | `Term:::Definition`                            | Language learning (Target ⇄ Native), synonyms        |
| **Multi-Line**        | `Question`<br>`?`<br>`Answer`                  | Explanations, code snippets, numbered lists          |
| **Cloze Deletion**    | `Text ==gap== text`                            | Definitions, formulas, keywords in context           |
| **Collapsible Block** | `<details><summary>...</summary>...</details>` | Universal viewing in any standard Markdown previewer |
