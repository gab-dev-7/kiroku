# kiroku-tui

**kiroku** (記録) — _Japanese for "record", "document", or "archive"._

A simple, terminal-based personal journaling and note-taking tool written in Rust.

kiroku-tui helps you manage a collection of markdown notes directly from your terminal. It stores your notes in `~/kiroku` and integrates with Git for easy synchronization.

## Features

- **Terminal Interface**: Clean TUI built with `ratatui`.
- **Folder Support**: Organize your notes into directories and navigate them with a file browser.
- **Fuzzy Search**: Quickly find notes by title across all folders.
- **Content Search**: Deep search within the body of your notes.
- **Tag Search**: Filter notes by tags defined in YAML frontmatter.
- **Note Renaming**: Rename existing notes directly within the app.
- **Smart Sorting**: Toggle between sorting by Date, Name, or Size.
- **External Editor**: Opens notes in your preferred editor (Vim, Nano, VS Code, etc.).
- **Smart Git Sync**: Built-in command to add, commit, and push changes. Skips redundant network calls if up-to-date.
- **File Watching**: Automatically updates the list when files are changed externally.
- **Auto-Sync on Exit**: Optional setting to automatically sync with Git when quitting.
- **Theming**: Fully customizable color schemes with built-in theme cycling.
- **Clipboard Integration**: Copy note content or file paths directly to your clipboard.

## Installation

Ensure you have [Rust and Cargo](https://rustup.rs/) installed.

### From Crates.io (Recommended)

```bash
cargo install kiroku-tui
```

### From Git

```bash
cargo install --git https://github.com/gab-dev-7/kiroku
```

> **Note:** After installation, ensure that `~/.cargo/bin` is in your `PATH` environment variable to run `kiroku` from any directory.

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

### Navigation Modes

**Browser Mode (Default)**
View your notes and folders hierarchically.
- Use `h` and `l` to navigate in and out of directories.
- Use `f` to create new folders.

**Search Mode**
When you start searching (`/`, `#`, `?`), the view switches to a flat list of all matching notes, regardless of their folder.

### Keybindings

**Normal Mode**

- `F1`: Open help popup
- `n`: Create a new note
- `f`: Create a new folder
- `Enter` / `l`: Edit selected note or Enter folder
- `Backspace` / `h`: Go up a directory
- `r`: Rename the selected item
- `d`: Delete the selected item (prompts for confirmation)
- `s`: Cycle sort mode (Date, Name, Size)
- `t`: Cycle built-in themes (Default -> Gruvbox -> Tokyo Night)
- `g`: Sync with Git (add, commit, push)
- `/`: Enter title search mode
- `?`: Enter content search mode
- `#`: Enter tag search mode
- `j` / `k`: Navigate down/up
- `Ctrl+j` / `Ctrl+k`: Scroll preview pane down/up
- `y`: Copy note content to clipboard
- `Y`: Copy note file path to clipboard
- `q`: Quit
- `F12`: Toggle debug logs

**Search Mode**

- Type to filter notes
- `Enter`: Keep current filter and return to list
- `Esc`: Clear search and return to browser view

### Using Tags

kiroku supports tagging notes using YAML frontmatter at the top of your markdown files.

```markdown
---
tags: [work, meeting, important]
---

# My Note Title

...
```

Use `#` to filter your notes by these tags.

## Configuration

You can configure kiroku by creating a file at `~/.config/kiroku/config.toml`.

**Example `config.toml`:**

```toml
# Command to open your editor.
# If omitted, defaults to $EDITOR environment variable or "vim".
editor_cmd = "nvim"

# Automatically sync with git when exiting the application.
auto_sync = false

# Default sort mode for notes ("Date", "Name", "Size").
sort_mode = "Date"

# Optional: Customize the color theme (hex codes)
[theme]
accent = "#89dceb"    # Key UI elements
selection = "#bb9af7" # Selected item
header = "#89b4fa"    # List headers
dim = "#6c7086"       # Metadata/dates
bold = "#f38ba8"      # Emphasized text
```

## Contributing

Contributions are welcome! Please check out [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to get started.

## License

MIT License. See [LICENSE](LICENSE) for details.
