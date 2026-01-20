use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

mod data;
use data::Note;

struct App {
    notes: Vec<Note>,
    list_state: ListState,
    status_msg: String,
    base_path: PathBuf,
}

impl App {
    fn new(notes: Vec<Note>, base_path: PathBuf) -> App {
        let mut state = ListState::default();
        if !notes.is_empty() {
            state.select(Some(0));
        }
        App {
            notes,
            list_state: state,
            status_msg: String::from("press 'n' for new note, 'enter' to edit, 'g' to sync"),
            base_path,
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.notes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.notes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

fn open_editor(base_path: &PathBuf, file_path: Option<&PathBuf>) -> io::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen)?;

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let mut cmd = Command::new(editor);
    cmd.current_dir(base_path);

    if let Some(path) = file_path {
        cmd.arg(path);
    }

    cmd.status()?;

    execute!(io::stdout(), EnterAlternateScreen)?;
    Ok(())
}

fn run_git_sync(base_path: &PathBuf) -> String {
    if !base_path.join(".git").exists() {
        return "not a git repo (run 'git init' in folder)".to_string();
    }

    let add = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(base_path)
        .output();
    if add.is_err() {
        return "git not installed?".to_string();
    }

    let _commit = Command::new("git")
        .args(["commit", "-m", "auto-sync from kiroku"])
        .current_dir(base_path)
        .output();

    let push = Command::new("git")
        .arg("push")
        .current_dir(base_path)
        .output();

    match push {
        Ok(output) => {
            if output.status.success() {
                "synced!".to_string()
            } else {
                "push failed (auth error?)".to_string()
            }
        }
        Err(_) => "git error".to_string(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("could not find home directory")?;
    let kiroku_path = home_dir.join("kiroku");

    if !kiroku_path.exists() {
        fs::create_dir_all(&kiroku_path)?;
        println!("created new notebook directory at {:?}", kiroku_path);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let path_str = kiroku_path.to_string_lossy().to_string();
    let notes = data::load_notes(&path_str);

    let mut app = App::new(notes, kiroku_path.clone());

    let mut should_quit = false;

    while !should_quit {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(f.area());

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(chunks[0]);

            let items: Vec<ListItem> = app
                .notes
                .iter()
                .map(|note| ListItem::new(note.title.clone()))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(" kiroku notes ")
                        .borders(Borders::ALL),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::REVERSED)
                        .fg(Color::Yellow),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, main_chunks[0], &mut app.list_state);

            let current_content = if let Some(i) = app.list_state.selected() {
                app.notes[i].content.as_str()
            } else {
                "press 'n' to create a new note."
            };

            let preview = Paragraph::new(current_content)
                .block(Block::default().title(" preview ").borders(Borders::ALL))
                .wrap(Wrap { trim: true });

            f.render_widget(preview, main_chunks[1]);

            let status = Paragraph::new(app.status_msg.as_str())
                .block(Block::default().borders(Borders::ALL).title(" status "));
            f.render_widget(status, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => should_quit = true,
                KeyCode::Char('j') => app.next(),
                KeyCode::Char('k') => app.previous(),
                KeyCode::Char('g') => {
                    app.status_msg = String::from("syncing...");
                    app.status_msg = run_git_sync(&app.base_path);
                }
                KeyCode::Char('n') => {
                    let _ = open_editor(&app.base_path, None);
                    let path_str = app.base_path.to_string_lossy().to_string();
                    app.notes = data::load_notes(&path_str);
                    terminal.clear()?;
                }
                KeyCode::Enter => {
                    if let Some(i) = app.list_state.selected() {
                        let path = app.notes[i].path.clone();
                        let _ = open_editor(&app.base_path, Some(&path));
                        let path_str = app.base_path.to_string_lossy().to_string();
                        app.notes = data::load_notes(&path_str);
                        terminal.clear()?;
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
