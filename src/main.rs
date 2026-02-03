use anyhow::Result;
use arboard::Clipboard;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use kiroku_tui::{
    app::{Action, App, InputMode},
    config, data,
    errors::KirokuError,
    events::{AppEvent, EventHandler},
    ops, ui,
};
use notify::{RecursiveMode, Watcher};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::fs;
use std::io;
use std::path::PathBuf;

// main entry point for the application
fn main() -> Result<()> {
    tui_logger::init_logger(log::LevelFilter::Info).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Info);

    // setup directory using cli arg or default
    let args: Vec<String> = std::env::args().collect();
    let kiroku_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("could not find home directory"))?;
        home_dir.join("kiroku")
    };

    // create notebook directory if missing
    if !kiroku_path.exists() {
        fs::create_dir_all(&kiroku_path)?;
        println!("created new notebook directory at {:?}", kiroku_path);
    }

    // load configuration
    let config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to load config: {}", e);
            config::Config::default()
        }
    };

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // load notes from directory
    let path_str = kiroku_path.to_string_lossy().to_string();
    let notes = match data::load_notes(&path_str) {
        Ok(n) => n,
        Err(e) => {
            log::error!("Failed to load notes: {}", e);
            vec![]
        }
    };
    let mut app = App::new(notes, kiroku_path.clone(), config);

    // setup event handler
    let events = EventHandler::new(250);

    // setup file watcher
    let tx = events.sender.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res
            && (event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove())
        {
            let _ = tx.send(AppEvent::FileChanged);
        }
    })?;
    watcher.watch(&kiroku_path, RecursiveMode::NonRecursive)?;

    // main loop
    while !app.should_quit {
        terminal.draw(|f| ui::ui(f, &mut app))?;

        match events.next()? {
            AppEvent::Input(key) => {
                let action = app.handle_input(key);
                match action {
                    Action::Quit => {
                        if app.config.auto_sync.unwrap_or(false) {
                            app.syncing = true;

                            events.pause();
                            let _ = disable_raw_mode();
                            let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                            let _ = terminal.show_cursor();

                            println!("Auto-syncing with git before exit...");
                            if let Err(e) = ops::run_git_sync(&app.base_path) {
                                log::error!("Auto-sync failed: {}", e);
                                println!("Auto-sync failed: {}", e);
                                std::thread::sleep(std::time::Duration::from_secs(2));
                            }
                        }
                        app.quit()
                    }
                    Action::ToggleLogs => {
                        app.show_logs = !app.show_logs;
                    }
                    Action::Sync => {
                        if !app.syncing {
                            app.syncing = true;

                            events.pause();
                            std::thread::sleep(std::time::Duration::from_millis(300));

                            // suspend tui for shell commands
                            let _ = disable_raw_mode();
                            let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                            let _ = terminal.show_cursor();

                            use std::io::Write;
                            let _ = io::stdout().flush();

                            println!("Syncing with git...");
                            println!("Repository path: {:?}", app.base_path);
                            println!("(If prompted for password, input will be hidden)");

                            // run git sync
                            let result =
                                ops::run_git_sync(&app.base_path).map_err(|e| e.to_string());

                            let _ = execute!(terminal.backend_mut(), EnterAlternateScreen);
                            let _ = enable_raw_mode();
                            let _ = terminal.clear();

                            events.resume();

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
                    }
                    Action::NewNote => {
                        app.input_mode = InputMode::Editing;
                        app.input.clear();
                        app.status_msg = String::from("Enter filename: ");
                    }
                    Action::NewFolder => {
                        app.input_mode = InputMode::CreatingFolder;
                        app.input.clear();
                        app.status_msg = String::from("Enter folder name: ");
                    }
                    Action::RenameNote => {
                        if let Some(i) = app.list_state.selected() {
                            let title = if !app.search_query.is_empty() {
                                if i < app.notes.len() {
                                    Some(app.notes[i].title.clone())
                                } else {
                                    None
                                }
                            } else if i < app.fs_items.len() {
                                match &app.fs_items[i] {
                                    data::FileSystemItem::Note(n) => Some(n.title.clone()),
                                    data::FileSystemItem::Folder(p) => {
                                        Some(p.file_name().unwrap().to_string_lossy().to_string())
                                    }
                                }
                            } else {
                                None
                            };

                            if let Some(t) = title {
                                app.input_mode = InputMode::Renaming;
                                app.input = t;
                                app.status_msg = String::from("Rename item: ");
                            }
                        }
                    }
                    Action::DeleteNote => {
                        if let Some(i) = app.list_state.selected() {
                            let name = if !app.search_query.is_empty() {
                                if i < app.notes.len() {
                                    Some(app.notes[i].title.clone())
                                } else {
                                    None
                                }
                            } else if i < app.fs_items.len() {
                                match &app.fs_items[i] {
                                    data::FileSystemItem::Note(n) => Some(n.title.clone()),
                                    data::FileSystemItem::Folder(p) => {
                                        Some(p.file_name().unwrap().to_string_lossy().to_string())
                                    }
                                }
                            } else {
                                None
                            };

                            if let Some(n) = name {
                                app.input_mode = InputMode::ConfirmDelete;
                                app.status_msg = format!("Delete '{}'? (y/n)", n);
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
                        InputMode::Editing => {
                            if !app.input.trim().is_empty() {
                                // create relative to current_dir
                                let target_path = app.base_path.join(&app.current_dir);
                                match ops::create_note(&target_path, &app.input) {
                                    Ok(path) => {
                                        events.pause();
                                        if let Err(e) = ops::open_editor(
                                            &app.base_path,
                                            Some(&path),
                                            app.config.editor_cmd.as_deref(),
                                        ) {
                                            log::error!("Failed to open editor: {}", e);
                                        }
                                        events.resume();
                                        app.input_mode = InputMode::Normal;
                                        app.status_msg = String::from("Note created.");
                                        // refresh view to show new file
                                        app.refresh_fs_view();
                                        terminal.clear()?;
                                    }
                                    Err(e) => {
                                        app.status_msg = format!("Error: {}", e);
                                    }
                                }
                            }
                        }
                        InputMode::Renaming => {
                            if let Some(i) = app.list_state.selected()
                                && !app.input.trim().is_empty()
                            {
                                let old_path = if !app.search_query.is_empty() {
                                    if i < app.notes.len() {
                                        Some(app.notes[i].path.clone())
                                    } else {
                                        None
                                    }
                                } else if i < app.fs_items.len() {
                                    match &app.fs_items[i] {
                                        data::FileSystemItem::Note(n) => Some(n.path.clone()),
                                        data::FileSystemItem::Folder(p) => Some(p.clone()),
                                    }
                                } else {
                                    None
                                };

                                if let Some(path) = old_path {
                                    match ops::rename_note(&path, &app.input) {
                                        Ok(_) => {
                                            app.input_mode = InputMode::Normal;
                                            app.status_msg = String::from("Item renamed.");
                                            app.refresh_fs_view();
                                        }
                                        Err(e) => {
                                            app.status_msg = format!("Rename error: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        InputMode::ConfirmDelete => {
                            if let Some(i) = app.list_state.selected() {
                                let path_to_delete = if !app.search_query.is_empty() {
                                    if i < app.notes.len() {
                                        Some(app.notes[i].path.clone())
                                    } else {
                                        None
                                    }
                                } else if i < app.fs_items.len() {
                                    match &app.fs_items[i] {
                                        data::FileSystemItem::Note(n) => Some(n.path.clone()),
                                        data::FileSystemItem::Folder(p) => Some(p.clone()),
                                    }
                                } else {
                                    None
                                };

                                if let Some(path) = path_to_delete {
                                    let res = if path.is_dir() {
                                        fs::remove_dir_all(&path).map_err(KirokuError::Io)
                                    } else {
                                        ops::delete_note(&path)
                                    };

                                    if let Err(e) = res {
                                        app.status_msg = format!("Delete error: {}", e);
                                    } else {
                                        app.status_msg = String::from("Item deleted.");
                                        app.refresh_fs_view();
                                    }
                                }
                            }
                            app.input_mode = InputMode::Normal;
                        }
                        InputMode::CreatingFolder => {
                            if !app.input.trim().is_empty() {
                                let target_path = app.base_path.join(&app.current_dir);
                                match ops::create_folder(&target_path, &app.input) {
                                    Ok(_) => {
                                        app.input_mode = InputMode::Normal;
                                        app.status_msg = String::from("Folder created.");
                                        app.refresh_fs_view();
                                    }
                                    Err(e) => {
                                        app.status_msg = format!("Error: {}", e);
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Action::CancelInput => {
                        app.input_mode = InputMode::Normal;
                        app.input.clear();
                        app.status_msg = String::from("Cancelled.");
                    }
                    Action::EditNote => {
                        if let Some(i) = app.list_state.selected() {
                            let path = if !app.search_query.is_empty() {
                                if i < app.notes.len() {
                                    Some(app.notes[i].path.clone())
                                } else {
                                    None
                                }
                            } else if i < app.fs_items.len() {
                                match &app.fs_items[i] {
                                    data::FileSystemItem::Note(n) => Some(n.path.clone()),
                                    data::FileSystemItem::Folder(_) => None, // cannot edit folder
                                }
                            } else {
                                None
                            };

                            if let Some(p) = path {
                                events.pause();
                                if let Err(e) = ops::open_editor(
                                    &app.base_path,
                                    Some(&p),
                                    app.config.editor_cmd.as_deref(),
                                ) {
                                    log::error!("Failed to open editor for {:?}: {}", p, e);
                                    app.status_msg = format!("Editor error: {}", e);
                                } else {
                                    terminal.clear()?;
                                }
                                events.resume();
                            }
                        }
                    }
                    Action::CopyContent => {
                        if let Some(i) = app.list_state.selected() {
                            let content = if !app.search_query.is_empty() {
                                if i < app.notes.len() {
                                    app.notes[i].content.clone()
                                } else {
                                    None
                                }
                            } else if i < app.fs_items.len() {
                                match &app.fs_items[i] {
                                    data::FileSystemItem::Note(n) => n.content.clone(),
                                    data::FileSystemItem::Folder(_) => None,
                                }
                            } else {
                                None
                            };

                            if let Some(c) = content {
                                // lazy init clipboard if missing
                                if app.clipboard.is_none() {
                                    match Clipboard::new() {
                                        Ok(cb) => app.clipboard = Some(cb),
                                        Err(e) => {
                                            log::warn!("Failed to re-init clipboard: {}", e)
                                        }
                                    }
                                }

                                if let Some(cb) = &mut app.clipboard {
                                    if let Err(e) = cb.set_text(c) {
                                        app.status_msg = format!("Copy error: {}", e);
                                    } else {
                                        app.status_msg =
                                            String::from("Content copied to clipboard.");
                                    }
                                } else {
                                    app.status_msg = String::from("Clipboard unavailable.");
                                }
                            } else {
                                app.status_msg =
                                    String::from("Note content not loaded or item is folder.");
                            }
                        }
                    }
                    Action::CopyPath => {
                        if let Some(i) = app.list_state.selected() {
                            let path_str = if !app.search_query.is_empty() {
                                if i < app.notes.len() {
                                    Some(app.notes[i].path.to_string_lossy().to_string())
                                } else {
                                    None
                                }
                            } else if i < app.fs_items.len() {
                                match &app.fs_items[i] {
                                    data::FileSystemItem::Note(n) => {
                                        Some(n.path.to_string_lossy().to_string())
                                    }
                                    data::FileSystemItem::Folder(p) => {
                                        Some(p.to_string_lossy().to_string())
                                    }
                                }
                            } else {
                                None
                            };

                            if let Some(p) = path_str {
                                if app.clipboard.is_none() {
                                    match Clipboard::new() {
                                        Ok(cb) => app.clipboard = Some(cb),
                                        Err(e) => log::warn!("Failed to re-init clipboard: {}", e),
                                    }
                                }

                                if let Some(cb) = &mut app.clipboard {
                                    if let Err(e) = cb.set_text(p) {
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
                    Action::ScrollUp => {
                        app.preview_scroll = app.preview_scroll.saturating_sub(1);
                    }
                    Action::ScrollDown => {
                        app.preview_scroll = app.preview_scroll.saturating_add(1);
                    }
                    Action::CycleSort => {
                        app.sort_mode = app.sort_mode.next();
                        app.sort_notes();
                        app.save_config();
                    }
                    Action::CycleTheme => {
                        app.cycle_theme();
                    }
                    Action::None => {}
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

    // restore terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
