use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
};
use std::fs;
use std::io;

mod app;
mod data;
mod errors;
mod events;
mod ops;
mod ui;

use app::{App, Action};
use events::{EventHandler, AppEvent};

fn main() -> Result<()> {
    // Setup directory
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("could not find home directory"))?;
    let kiroku_path = home_dir.join("kiroku");

    if !kiroku_path.exists() {
        fs::create_dir_all(&kiroku_path)?;
        println!("created new notebook directory at {:?}", kiroku_path);
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load initial data
    let path_str = kiroku_path.to_string_lossy().to_string();
    let notes = data::load_notes(&path_str);
    let mut app = App::new(notes, kiroku_path.clone());

    // Setup events
    let events = EventHandler::new(250); // 250ms tick

    while !app.should_quit {
        terminal.draw(|f| ui::ui(f, &mut app))?;

        match events.next()? {
            AppEvent::Input(key) => {
                let action = app.handle_input(key);
                match action {
                    Action::Quit => app.quit(),
                    Action::Sync => {
                        app.status_msg = String::from("syncing...");
                        terminal.draw(|f| ui::ui(f, &mut app))?;
                        
                        match ops::run_git_sync(&app.base_path) {
                             Ok(msg) => app.status_msg = msg,
                             Err(e) => app.status_msg = format!("Sync error: {}", e),
                        }
                    }
                    Action::NewNote => {
                        if let Err(e) = ops::open_editor(&app.base_path, None) {
                             app.status_msg = format!("Editor error: {}", e);
                        } else {
                             let path_str = app.base_path.to_string_lossy().to_string();
                             app.notes = data::load_notes(&path_str);
                             terminal.clear()?;
                        }
                    }
                    Action::EditNote => {
                        if let Some(i) = app.list_state.selected() {
                             if i < app.notes.len() {
                                 let path = app.notes[i].path.clone();
                                 if let Err(e) = ops::open_editor(&app.base_path, Some(&path)) {
                                     app.status_msg = format!("Editor error: {}", e);
                                 } else {
                                     let path_str = app.base_path.to_string_lossy().to_string();
                                     app.notes = data::load_notes(&path_str);
                                     terminal.clear()?;
                                 }
                             }
                        }
                    }
                    Action::None => {}
                }
            }
            AppEvent::Tick => {
                app.tick();
            }
        }
    }

    // Teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}