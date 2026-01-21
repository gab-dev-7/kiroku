# kiroku

**kiroku** (記録) — _Japanese for "record", "document", or "archive"._

A simple, terminal-based personal journaling and note-taking tool written in Rust.

kiroku helps you manage a collection of markdown notes directly from your terminal. It stores your notes in `~/kiroku` and integrates with Git for easy synchronization.

## Features

- **Terminal Interface**: Clean TUI built with `ratatui`.
- **Fuzzy Search**: Quickly find notes by title.
- **External Editor**: Opens notes in your preferred editor (Vim, Nano, VS Code, etc.).
- **Git Sync**: Built-in command to add, commit, and push changes to a remote repository.
- **File Watching**: Automatically updates the list when files are changed externally.
- **Clipboard Integration**: Copy note content or file paths directly to your clipboard.

## Installation

Ensure you have [Rust and Cargo](https://rustup.rs/) installed.

### From Git (Recommended for users)

You can install `kiroku` directly from the repository:

```bash
cargo install --git https://github.com/gab-dev-7/kiroku
```

### From Source (For development)

If you have cloned the repository, you can install it locally:

```bash
cargo install --path .
```

## Usage

Run the application:

```bash
kiroku
```

On the first run, it will create a `~/kiroku` directory. You can initialize a git repository there if you want to use the sync feature:

```bash
cd ~/kiroku
git init
# Add your remote...
```

### Keybindings

**Normal Mode**

- `n`: Create a new note
- `Enter`: Edit the selected note
- `d`: Delete the selected note
- `g`: Sync with Git (add, commit, push)
- `/`: Enter search mode
- `j` / `k`: Navigate up/down
- `y`: Copy note content to clipboard
- `Y`: Copy note file path to clipboard
- `q`: Quit
- `F12`: Toggle debug logs

**Search Mode**

- Type to filter notes
- `Enter`: Keep current filter and return to list
- `Esc`: Clear search and return to list

## Configuration

You can configure kiroku by creating a file at `~/.config/kiroku/config.toml`.

**Example `config.toml`:**

```toml
# Command to open your editor.
# If omitted, defaults to $EDITOR environment variable or "vim".
editor_cmd = "nvim"

# Auto-sync is currently reserved for future use
auto_sync = false
```

