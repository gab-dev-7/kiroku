use crate::data::Note;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::path::PathBuf;

pub enum Action {
    None,
    Quit,
    Sync,
    NewNote,
    EditNote,
    ToggleLogs,
}

pub struct App {
    pub notes: Vec<Note>,
    pub list_state: ListState,
    pub status_msg: String,
    pub base_path: PathBuf,
    pub should_quit: bool,
    pub show_logs: bool,
}

impl App {
    pub fn new(notes: Vec<Note>, base_path: PathBuf) -> App {
        let mut state = ListState::default();
        if !notes.is_empty() {
            state.select(Some(0));
        }
        App {
            notes,
            list_state: state,
            status_msg: String::from("press 'n' for new note, 'enter' to edit, 'g' to sync"),
            base_path,
            should_quit: false,
            show_logs: false,
        }
    }

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn tick(&mut self) {
        // Time-based updates (spinners, etc.) will go here
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('j') => {
                self.next();
                Action::None
            }
            KeyCode::Char('k') => {
                self.previous();
                Action::None
            }
            KeyCode::Char('g') => Action::Sync,
            KeyCode::Char('n') => Action::NewNote,
            KeyCode::Enter => Action::EditNote,
            KeyCode::F(12) => Action::ToggleLogs,
            _ => Action::None,
        }
    }
}

