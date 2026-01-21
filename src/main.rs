use anyhow::Result;
use arboard::Clipboard;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify::{RecursiveMode, Watcher};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::fs;
use std::io;

mod app;
mod config;
mod data;
mod errors;
mod events;
mod ops;
mod ui;

use app::{Action, App};
use events::{AppEvent, EventHandler};

fn main() -> Result<()> {
    tui_logger::init_logger(log::LevelFilter::Info).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Info);

    // Setup directory
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("could not find home directory"))?;
    let kiroku_path = home_dir.join("kiroku");

    if !kiroku_path.exists() {
        fs::create_dir_all(&kiroku_path)?;
        println!("created new notebook directory at {:?}", kiroku_path);
    }

    // Load config
    let config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to load config: {}", e);
            config::Config::default()
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load initial data
    let path_str = kiroku_path.to_string_lossy().to_string();
    let notes = match data::load_notes(&path_str) {
        Ok(n) => n,
        Err(e) => {
            log::error!("Failed to load notes: {}", e);
            vec![]
        }
    };
    let mut app = App::new(notes, kiroku_path.clone(), config);

    // Setup events
    let events = EventHandler::new(250);

    // Setup file watcher
    let tx = events.sender.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                let _ = tx.send(AppEvent::FileChanged);
            }
        }
    })?;
    watcher.watch(&kiroku_path, RecursiveMode::NonRecursive)?;

    while !app.should_quit {
        terminal.draw(|f| ui::ui(f, &mut app))?;

        match events.next()? {
            AppEvent::Input(key) => {
                let action = app.handle_input(key);
                match action {
                    Action::Quit => app.quit(),
                    Action::ToggleLogs => {
                        app.show_logs = !app.show_logs;
                    }
                    Action::Sync => {
                        if !app.syncing {
                            app.syncing = true;
                            app.status_msg = String::from("syncing...");

                            let tx = events.sender.clone();
                            let base_path = app.base_path.clone();

                            std::thread::spawn(move || {
                                let result =
                                    ops::run_git_sync(&base_path).map_err(|e| e.to_string());
                                let _ = tx.send(AppEvent::SyncFinished(result));
                            });
                        }
                    }
                    Action::NewNote => {
                        app.input_mode = app::InputMode::Editing;
                        app.input.clear();
                        app.status_msg = String::from("Enter filename: ");
                    }
                    Action::DeleteNote => {
                        if let Some(i) = app.list_state.selected() {
                            if i < app.notes.len() {
                                app.input_mode = app::InputMode::ConfirmDelete;
                                app.status_msg = format!("Delete '{}'? (y/n)", app.notes[i].title);
                            }
                        }
                    }
                    Action::EnterChar(c) => {
                        app.input.push(c);
                    }
                    Action::Backspace => {
                        app.input.pop();
                    }
                    Action::SubmitInput => match app.input_mode {
                        app::InputMode::Editing => {
                            if !app.input.trim().is_empty() {
                                match ops::create_note(&app.base_path, &app.input) {
                                    Ok(path) => {
                                        if let Err(e) = ops::open_editor(
                                            &app.base_path,
                                            Some(&path),
                                            app.config.editor_cmd.as_deref(),
                                        ) {
                                            log::error!("Failed to open editor: {}", e);
                                        }
                                        app.input_mode = app::InputMode::Normal;
                                        app.status_msg = String::from("Note created.");
                                        terminal.clear()?;
                                    }
                                    Err(e) => {
                                        app.status_msg = format!("Error: {}", e);
                                    }
                                }
                            }
                        }
                        app::InputMode::ConfirmDelete => {
                            if let Some(i) = app.list_state.selected() {
                                if i < app.notes.len() {
                                    if let Err(e) = ops::delete_note(&app.notes[i].path) {
                                        app.status_msg = format!("Delete error: {}", e);
                                    } else {
                                        app.status_msg = String::from("Note deleted.");
                                    }
                                }
                            }
                            app.input_mode = app::InputMode::Normal;
                        }
                        _ => {}
                    },
                    Action::CancelInput => {
                        app.input_mode = app::InputMode::Normal;
                        app.input.clear();
                        app.status_msg = String::from("Cancelled.");
                    }
                    Action::EditNote => {
                        if let Some(i) = app.list_state.selected() {
                            if i < app.notes.len() {
                                let path = app.notes[i].path.clone();
                                if let Err(e) = ops::open_editor(
                                    &app.base_path,
                                    Some(&path),
                                    app.config.editor_cmd.as_deref(),
                                ) {
                                    log::error!("Failed to open editor for {:?}: {}", path, e);
                                    app.status_msg = format!("Editor error: {}", e);
                                } else {
                                    // notify will handle reloading
                                    terminal.clear()?;
                                }
                            }
                        }
                    }
                    Action::CopyContent => {
                        if let Some(i) = app.list_state.selected() {
                            if i < app.notes.len() {
                                let note = &app.notes[i];
                                if let Some(content) = &note.content {
                                    // Lazy init clipboard if missing
                                    if app.clipboard.is_none() {
                                        match Clipboard::new() {
                                            Ok(cb) => app.clipboard = Some(cb),
                                            Err(e) => {
                                                log::warn!("Failed to re-init clipboard: {}", e)
                                            }
                                        }
                                    }

                                    if let Some(cb) = &mut app.clipboard {
                                        if let Err(e) = cb.set_text(content.clone()) {
                                            app.status_msg = format!("Copy error: {}", e);
                                        } else {
                                            app.status_msg =
                                                String::from("Content copied to clipboard.");
                                        }
                                    } else {
                                        app.status_msg = String::from("Clipboard unavailable.");
                                    }
                                } else {
                                    app.status_msg = String::from("Note content not loaded.");
                                }
                            }
                        }
                    }
                    Action::CopyPath => {
                        if let Some(i) = app.list_state.selected() {
                            if i < app.notes.len() {
                                let path = app.notes[i].path.to_string_lossy().to_string();

                                // Lazy init clipboard if missing
                                if app.clipboard.is_none() {
                                    match Clipboard::new() {
                                        Ok(cb) => app.clipboard = Some(cb),
                                        Err(e) => log::warn!("Failed to re-init clipboard: {}", e),
                                    }
                                }

                                if let Some(cb) = &mut app.clipboard {
                                    if let Err(e) = cb.set_text(path) {
                                        app.status_msg = format!("Copy error: {}", e);
                                    } else {
                                        app.status_msg = String::from("Path copied to clipboard.");
                                    }
                                } else {
                                    app.status_msg = String::from("Clipboard unavailable.");
                                }
                            }
                        }
                    }
                    Action::None => {}
                }
            }
            AppEvent::SyncFinished(result) => {
                app.syncing = false;
                match result {
                    Ok(msg) => {
                        log::info!("Sync successful: {}", msg);
                        app.status_msg = msg;
                    }
                    Err(e) => {
                        log::error!("Sync failed: {}", e);
                        app.status_msg = format!("Sync error: {}", e);
                    }
                }
            }
            AppEvent::Tick => {
                app.tick();
            }
            AppEvent::FileChanged => {
                let path_str = app.base_path.to_string_lossy().to_string();
                if let Ok(notes) = data::load_notes(&path_str) {
                    app.all_notes = notes;
                    app.update_search();
                }
            }
        }
    }

    // Teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
