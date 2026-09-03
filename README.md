_Flashcards Format_
flashcards.md
`mdflash`: opens with last opened directory,

`mdflash "<path-to-direcory>"` opens directory

`mdflash "<path-to-file>"` opens flashcards

```markdown
// Metadata
---

tags:

- <tag1>[string?]
- <tag2>[string?]
- <tag3>[string?]
  <date>["YYYY-MM-DD"]
  <author>[string?]

---

# <deck_title>[string]

## <subsection>[string?]

// normal question-answer
<question>[string]::<answer>[string]

// definition: meaning
<question>[defenition]:::<answer>[meaning]

// multiline-answer-block
<question>[String]
?
<multiline-answer-1>
<multiline-answer-2>
<multiline-answer-3>

// Fill-in
<text_before>[string] ==<solution>== <text_after>[string].
```
