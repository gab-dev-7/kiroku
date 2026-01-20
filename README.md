# kiroku

**kiroku** (記録) — _Japanese for "record", "document", or "archive"._

A minimal, terminal-based note management tool written in Rust.

## philosophy

The name _Kiroku_ implies a permanent, official record. This tool is built on the idea that your notes should be treated as a durable archive, not a temporary scratchpad.

It adheres to three principles:

1.  **Plain Text:** Data lives in standard Markdown files. No databases, no vendor lock-in.
2.  **Git Backed:** Your history is preserved. A single keystroke secures your data.
3.  **Editor Agnostic:** The tool handles the organization; your preferred editor handles the writing.

## features

- **Split-Pane Interface:** Browse a list of notes while previewing contents.
- **Git Integration:** Press `g` to automatically stage, commit, and push changes.
- **Editor Support:** Seamlessly opens your `$EDITOR` (vim, nvim, helix, nano).
- **Time Sorting:** Notes are automatically ordered by modification time (newest first).

## installation

### prerequisites

- Rust and Cargo
- Git
- A terminal text editor

## usage

| Key       | Action                             |
| --------- | ---------------------------------- |
| `j` / `k` | Navigate the note list             |
| `enter`   | Open the selected note for editing |
| `n`       | Create a new note                  |
| `g`       | Sync with Git (add, commit, push)  |
| `q`       | Quit application                   |

## configuration

**Storage**
By default, the application reads and writes to:
`~/kiroku/`

## development

This is a learning project for Rust and TUI architecture. Contributions are welcome.
