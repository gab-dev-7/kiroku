use crate::config::Config;
use crate::data::{self, Note};
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::widgets::ListState;
use std::collections::VecDeque;
use std::path::PathBuf;

pub enum Action {
    None,
    Quit,
    Sync,
    NewNote,
    EditNote,
    DeleteNote,
    CopyContent,
    CopyPath,
    ToggleLogs,
    EnterChar(char),
    Backspace,
    SubmitInput,
    CancelInput,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    ConfirmDelete,
    Search,
}

pub struct App {
    pub notes: Vec<Note>,
    pub all_notes: Vec<Note>,
    pub list_state: ListState,
    pub status_msg: String,
    pub base_path: PathBuf,
    pub config: Config,
    pub should_quit: bool,
    pub show_logs: bool,
    pub recent_indices: VecDeque<usize>,
    pub input: String,
    pub search_query: String,
    pub input_mode: InputMode,
    pub syncing: bool,
    pub spinner_index: usize,
    pub clipboard: Option<Clipboard>,
}

impl App {
    pub fn new(notes: Vec<Note>, base_path: PathBuf, config: Config) -> App {
        let state = ListState::default();
        let all_notes = notes.clone();

        let clipboard = match Clipboard::new() {
            Ok(cb) => Some(cb),
            Err(e) => {
                log::warn!("Failed to initialize clipboard: {}", e);
                None
            }
        };

        let mut app = App {
            notes,
            all_notes,
            list_state: state,
            status_msg: String::from(
                " 'n' for new note, 'enter' to edit, 'g' to sync, 'd' to delete, '/' to search",
            ),
            base_path,
            config,
            should_quit: false,
            show_logs: false,
            recent_indices: VecDeque::with_capacity(10),
            input: String::new(),
            search_query: String::new(),
            input_mode: InputMode::Normal,
            syncing: false,
            spinner_index: 0,
            clipboard,
        };

        if !app.notes.is_empty() {
            app.list_state.select(Some(0));
            app.load_note_content(0);
        }
        app
    }

    pub fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.notes = self.all_notes.clone();
        } else {
            let matcher = SkimMatcherV2::default();
            let mut matches: Vec<(&Note, i64)> = self
                .all_notes
                .iter()
                .filter_map(|note| {
                    matcher
                        .fuzzy_match(&note.title, &self.search_query)
                        .map(|score| (note, score))
                })
                .collect();

            matches.sort_by(|a, b| b.1.cmp(&a.1));
            self.notes = matches.into_iter().map(|(n, _)| n.clone()).collect();
        }

        // Reset selection
        if !self.notes.is_empty() {
            self.list_state.select(Some(0));
            self.load_note_content(0);
        } else {
            self.list_state.select(None);
        }
    }

    pub fn load_note_content(&mut self, index: usize) {
        if index >= self.notes.len() {
            return;
        }

        if self.notes[index].content.is_none() {
            match data::read_note_content(&self.notes[index].path) {
                Ok(content) => {
                    self.notes[index].content = Some(content);
                    // Add to cache
                    self.recent_indices.push_back(index);
                    if self.recent_indices.len() > 10 {
                        if let Some(old_idx) = self.recent_indices.pop_front() {
                            // Don't clear if it's currently selected or still in recent list
                            if Some(old_idx) != self.list_state.selected()
                                && !self.recent_indices.contains(&old_idx)
                            {
                                self.notes[old_idx].content = None;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to load note content: {}", e);
                }
            }
        } else {
            // Already loaded, just move to back of LRU
            self.recent_indices.retain(|&i| i != index);
            self.recent_indices.push_back(index);
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
        self.load_note_content(i);
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
        self.load_note_content(i);
    }

    pub fn tick(&mut self) {
        if self.syncing {
            self.spinner_index = (self.spinner_index + 1) % 4;
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Action {
        match self.input_mode {
            InputMode::Normal => match key.code {
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
                KeyCode::Char('d') => Action::DeleteNote,
                KeyCode::Char('y') => Action::CopyContent,
                KeyCode::Char('Y') => Action::CopyPath,
                KeyCode::Char('/') => {
                    self.input_mode = InputMode::Search;
                    self.search_query.clear();
                    self.status_msg = String::from("Search: ");
                    Action::None
                }
                KeyCode::Enter => Action::EditNote,
                KeyCode::F(12) => Action::ToggleLogs,
                _ => Action::None,
            },
            InputMode::Editing => match key.code {
                KeyCode::Enter => Action::SubmitInput,
                KeyCode::Esc => Action::CancelInput,
                KeyCode::Backspace => Action::Backspace,
                KeyCode::Char(c) => Action::EnterChar(c),
                _ => Action::None,
            },
            InputMode::ConfirmDelete => match key.code {
                KeyCode::Char('y') => Action::SubmitInput,
                KeyCode::Char('n') | KeyCode::Esc => Action::CancelInput,
                _ => Action::None,
            },
            InputMode::Search => match key.code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    self.status_msg = String::from("Filter active. Esc to clear.");
                    Action::None
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.search_query.clear();
                    self.update_search();
                    self.status_msg = String::from(
                        "press 'n' for new note, 'enter' to edit, 'g' to sync, 'd' to delete, '/' to search",
                    );
                    Action::None
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.update_search();
                    self.status_msg = format!("Search: {}", self.search_query);
                    Action::None
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.update_search();
                    self.status_msg = format!("Search: {}", self.search_query);
                    Action::None
                }
                _ => Action::None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn create_test_note(title: &str) -> Note {
        Note {
            path: PathBuf::from(format!("{}.md", title)),
            title: title.to_string(),
            content: Some("content".to_string()),
            last_modified: SystemTime::now(),
            size: 100,
        }
    }

    #[test]
    fn test_search_filtering() {
        let notes = vec![
            create_test_note("alpha"),
            create_test_note("beta"),
            create_test_note("gamma"),
            create_test_note("apple"),
        ];

        let mut app = App::new(notes, PathBuf::from("/tmp"), Config::default());

        app.search_query = "ap".to_string();
        app.update_search();

        assert_eq!(app.notes.len(), 2);
        assert!(app.notes.iter().any(|n| n.title == "alpha"));
        assert!(app.notes.iter().any(|n| n.title == "apple"));

        app.search_query = "bet".to_string();
        app.update_search();

        assert_eq!(app.notes.len(), 1);
        assert_eq!(app.notes[0].title, "beta");

        app.search_query = "".to_string();
        app.update_search();
        assert_eq!(app.notes.len(), 4);
    }
}
