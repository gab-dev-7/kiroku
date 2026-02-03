use crate::config::Config;
use crate::data::{self, Note};
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::style::Color;
use ratatui::widgets::ListState;
use std::collections::VecDeque;
use std::path::PathBuf;

pub struct ThemeColors {
    pub accent: Color,
    pub selection: Color,
    pub header: Color,
    pub dim: Color,
    pub bold: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            accent: Color::Rgb(137, 220, 235),
            selection: Color::Rgb(187, 154, 247),
            dim: Color::Rgb(108, 112, 134),
            header: Color::Rgb(137, 180, 250),
            bold: Color::Rgb(243, 139, 168),
        }
    }
}

pub enum Action {
    None,
    Quit,
    Sync,
    NewNote,
    NewFolder,
    EditNote,
    DeleteNote,
    RenameNote,
    CopyContent,
    CopyPath,
    ToggleLogs,
    EnterChar(char),
    Backspace,
    SubmitInput,
    CancelInput,
    ScrollUp,
    ScrollDown,
    CycleSort,
    CycleTheme,
}

#[derive(PartialEq, Clone, Copy)]
pub enum SortMode {
    Date,
    Name,
    Size,
}

impl SortMode {
    pub fn next(&self) -> Self {
        match self {
            SortMode::Date => SortMode::Name,
            SortMode::Name => SortMode::Size,
            SortMode::Size => SortMode::Date,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SortMode::Date => "Date",
            SortMode::Name => "Name",
            SortMode::Size => "Size",
        }
    }
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    CreatingFolder,
    Renaming,
    ConfirmDelete,
    Search,
    ContentSearch,
    TagSearch,
    Help,
}

// main app state
pub struct App {
    pub notes: Vec<Note>,
    pub all_notes: Vec<Note>,
    pub fs_items: Vec<data::FileSystemItem>,
    pub current_dir: PathBuf,
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
    pub preview_scroll: u16,
    pub theme: ThemeColors,
    pub sort_mode: SortMode,
}

impl App {
    // init app state
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

        let initial_sort = if let Some(ref s) = config.sort_mode {
            match s.as_str() {
                "Name" => SortMode::Name,
                "Size" => SortMode::Size,
                _ => SortMode::Date,
            }
        } else {
            SortMode::Date
        };

        let mut app = App {
            notes,
            all_notes,
            fs_items: Vec::new(),
            current_dir: PathBuf::new(),
            list_state: state,
            status_msg: String::from(" Press 'h' for help "),
            base_path,
            config: config.clone(),
            should_quit: false,
            show_logs: false,
            recent_indices: VecDeque::with_capacity(10),
            input: String::new(),
            search_query: String::new(),
            input_mode: InputMode::Normal,
            syncing: false,
            spinner_index: 0,
            clipboard,
            preview_scroll: 0,
            theme: ThemeColors::default(),
            sort_mode: initial_sort,
        };

        if let Some(user_theme) = &config.theme {
            let parse = |s: &Option<String>, fallback: Color| -> Color {
                if let Some(hex) = s
                    && hex.starts_with('#')
                    && hex.len() == 7
                    && let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[1..3], 16),
                        u8::from_str_radix(&hex[3..5], 16),
                        u8::from_str_radix(&hex[5..7], 16),
                    )
                {
                    return Color::Rgb(r, g, b);
                }
                fallback
            };

            app.theme.accent = parse(&user_theme.accent, app.theme.accent);
            app.theme.selection = parse(&user_theme.selection, app.theme.selection);
            app.theme.header = parse(&user_theme.header, app.theme.header);
            app.theme.dim = parse(&user_theme.dim, app.theme.dim);
            app.theme.bold = parse(&user_theme.bold, app.theme.bold);
        }

        app.sort_notes();
        app.refresh_fs_view();

        if !app.fs_items.is_empty() {
            app.list_state.select(Some(0));
        }
        app
    }

    // refresh item list from current dir
    pub fn refresh_fs_view(&mut self) {
        let target_dir = self.base_path.join(&self.current_dir);
        let items_res = data::load_all_items(&target_dir.to_string_lossy());

        if let Ok(mut items) = items_res {
            let current_depth = self.current_dir.components().count();

            items.retain(|item| {
                let path = match item {
                    data::FileSystemItem::Note(n) => &n.path,
                    data::FileSystemItem::Folder(p) => p,
                };

                let rel_path = path.strip_prefix(&self.base_path).unwrap_or(path);

                if !rel_path.starts_with(&self.current_dir) {
                    return false;
                }

                let rel_depth = rel_path.components().count();
                rel_depth == current_depth + 1
            });

            self.fs_items = items;
        }
    }

    // sort notes
    pub fn sort_notes(&mut self) {
        if !self.search_query.is_empty() {
            return;
        }

        match self.sort_mode {
            SortMode::Date => {
                self.notes
                    .sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
            }
            SortMode::Name => {
                self.notes
                    .sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
            }
            SortMode::Size => {
                self.notes.sort_by(|a, b| b.size.cmp(&a.size));
            }
        }
    }

    // fuzzy search notes by title
    pub fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.notes = self.all_notes.clone();
            self.sort_notes();
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

        // reset selection
        if !self.notes.is_empty() {
            self.list_state.select(Some(0));
            self.load_note_content(0);
        } else {
            self.list_state.select(None);
        }
    }

    // fuzzy search notes by tags
    pub fn update_tag_search(&mut self) {
        if self.search_query.is_empty() {
            self.notes = self.all_notes.clone();
            self.sort_notes();
        } else {
            let matcher = SkimMatcherV2::default();
            let mut matches: Vec<(&Note, i64)> = self
                .all_notes
                .iter()
                .filter_map(|note| {
                    // Check if any tag matches
                    let best_score = note
                        .tags
                        .iter()
                        .filter_map(|tag| matcher.fuzzy_match(tag, &self.search_query))
                        .max();

                    best_score.map(|score| (note, score))
                })
                .collect();

            matches.sort_by(|a, b| b.1.cmp(&a.1));
            self.notes = matches.into_iter().map(|(n, _)| n.clone()).collect();
        }

        // reset selection
        if !self.notes.is_empty() {
            self.list_state.select(Some(0));
            self.load_note_content(0);
        } else {
            self.list_state.select(None);
        }
    }

    // fuzzy search notes by content
    pub fn update_content_search(&mut self) {
        if self.search_query.is_empty() {
            self.notes = self.all_notes.clone();
            self.sort_notes();
        } else {
            let matcher = SkimMatcherV2::default();
            let query = self.search_query.to_lowercase();

            let mut matches: Vec<(&Note, i64)> = self
                .all_notes
                .iter()
                .filter_map(|note| {
                    let content = if let Some(ref c) = note.content {
                        Some(c.clone())
                    } else {
                        data::read_note_content(&note.path).ok()
                    };

                    if let Some(content) = content
                        && content.to_lowercase().contains(&query)
                    {
                        let title_score = matcher.fuzzy_match(&note.title, &query).unwrap_or(0);
                        return Some((note, 100 + title_score));
                    }
                    None
                })
                .collect();

            matches.sort_by(|a, b| b.1.cmp(&a.1));
            self.notes = matches.into_iter().map(|(n, _)| n.clone()).collect();
        }

        // reset selection
        if !self.notes.is_empty() {
            self.list_state.select(Some(0));
            self.load_note_content(0);
        } else {
            self.list_state.select(None);
        }
    }

    // load content with lru cache
    pub fn load_note_content(&mut self, index: usize) {
        if index >= self.notes.len() {
            return;
        }

        if self.notes[index].content.is_none() {
            match data::read_note_content(&self.notes[index].path) {
                Ok(content) => {
                    self.notes[index].content = Some(content);
                }
                Err(e) => {
                    log::error!("Failed to load note content: {}", e);
                    return;
                }
            }
        }

        // update cache (lru)
        self.recent_indices.retain(|&i| i != index);
        self.recent_indices.push_back(index);

        if self.recent_indices.len() > 10
            && let Some(old_idx) = self.recent_indices.pop_front()
            && Some(old_idx) != self.list_state.selected()
            && !self.recent_indices.contains(&old_idx)
        {
            self.notes[old_idx].content = None;
        }
    }

    pub fn next(&mut self) {
        if !self.search_query.is_empty() {
            if self.notes.is_empty() {
                return;
            }
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
        } else {
            // navigate fs_items
            if self.fs_items.is_empty() {
                return;
            }
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i >= self.fs_items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
            self.load_fs_item_content(i);
        }
        self.preview_scroll = 0;
    }

    pub fn previous(&mut self) {
        // if searching, navigate notes list
        if !self.search_query.is_empty() {
            if self.notes.is_empty() {
                return;
            }
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
        } else {
            // navigate fs_items
            if self.fs_items.is_empty() {
                return;
            }
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.fs_items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
            self.load_fs_item_content(i);
        }
        self.preview_scroll = 0;
    }

    // load content for file system items
    pub fn load_fs_item_content(&mut self, index: usize) {
        if index >= self.fs_items.len() {
            return;
        }

        match &self.fs_items[index] {
            data::FileSystemItem::Note(_note) => {}
            data::FileSystemItem::Folder(_) => {
                return;
            }
        }

        if let data::FileSystemItem::Note(ref mut n) = self.fs_items[index]
            && n.content.is_none()
            && let Ok(c) = data::read_note_content(&n.path)
        {
            n.content = Some(c);
        }
    }

    pub fn tick(&mut self) {
        if self.syncing {
            self.spinner_index = (self.spinner_index + 1) % 4;
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn save_config(&self) {
        let mut config = self.config.clone();
        config.sort_mode = Some(self.sort_mode.as_str().to_string());
        if let Err(e) = crate::config::save_config(&config) {
            log::error!("Failed to save config: {}", e);
        }
    }

    pub fn cycle_theme(&mut self) {
        // cycle through themes
        let current_accent = self.theme.accent;

        // identify current theme by accent
        // Default: 137, 220, 235 (#89dceb)
        // Gruvbox: 250, 189, 47  (#fabd2f)
        // Tokyo:   122, 162, 247 (#7aa2f7)

        let next_theme = if current_accent == Color::Rgb(137, 220, 235) {
            // Gruvbox
            ThemeColors {
                accent: Color::Rgb(250, 189, 47),    // Yellow
                selection: Color::Rgb(215, 153, 33), // Dark Yellow
                header: Color::Rgb(251, 73, 52),     // Red
                dim: Color::Rgb(168, 153, 132),      // Gray
                bold: Color::Rgb(254, 128, 25),      // Orange
            }
        } else if current_accent == Color::Rgb(250, 189, 47) {
            // Tokyo Night
            ThemeColors {
                accent: Color::Rgb(122, 162, 247),    // Blue
                selection: Color::Rgb(187, 154, 247), // Purple
                header: Color::Rgb(125, 207, 255),    // Cyan
                dim: Color::Rgb(86, 95, 137),         // Dark Blue/Gray
                bold: Color::Rgb(247, 118, 142),      // Red/Pink
            }
        } else {
            // Back to Default (Catppuccin Mocha-ish)
            ThemeColors::default()
        };

        self.theme = next_theme;
    }

    // handle input
    pub fn handle_input(&mut self, key: KeyEvent) -> Action {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Char('j')
                    if key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    Action::ScrollDown
                }
                KeyCode::Char('k')
                    if key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    Action::ScrollUp
                }
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
                KeyCode::Char('f') => Action::NewFolder,
                KeyCode::Char('d') => Action::DeleteNote,
                KeyCode::Char('r') => Action::RenameNote,
                KeyCode::Char('s') => Action::CycleSort,
                KeyCode::Char('t') => Action::CycleTheme,
                KeyCode::Char('y') => Action::CopyContent,
                KeyCode::Char('Y') => Action::CopyPath,
                KeyCode::Char('/') => {
                    self.input_mode = InputMode::Search;
                    self.search_query.clear();
                    self.status_msg = String::from("Search: ");
                    Action::None
                }
                KeyCode::Char('#') => {
                    self.input_mode = InputMode::TagSearch;
                    self.search_query.clear();
                    self.status_msg = String::from("Tag Search: ");
                    Action::None
                }
                KeyCode::Char('?') => {
                    self.input_mode = InputMode::ContentSearch;
                    self.search_query.clear();
                    self.status_msg = String::from("Content Search: ");
                    Action::None
                }
                KeyCode::F(1) => {
                    self.input_mode = InputMode::Help;
                    self.status_msg = String::from(" Help ");
                    Action::None
                }
                KeyCode::Char('h') | KeyCode::Backspace => {
                    if self.search_query.is_empty()
                        && self.current_dir.components().count() > 0
                        && let Some(parent) = self.current_dir.parent()
                    {
                        self.current_dir = parent.to_path_buf();
                        self.refresh_fs_view();
                        self.list_state.select(Some(0));
                        self.status_msg = format!("Dir: {}", self.current_dir.display());
                    }
                    Action::None
                }
                KeyCode::Char('l') | KeyCode::Enter => {
                    // check if selected item is folder
                    if self.search_query.is_empty() {
                        if let Some(i) = self.list_state.selected() {
                            if i < self.fs_items.len() {
                                match &self.fs_items[i] {
                                    data::FileSystemItem::Folder(path) => {
                                        // enter folder
                                        let rel_path =
                                            path.strip_prefix(&self.base_path).unwrap_or(path);
                                        self.current_dir = rel_path.to_path_buf();
                                        self.refresh_fs_view();
                                        self.list_state.select(Some(0));
                                        self.status_msg =
                                            format!("Dir: {}", self.current_dir.display());
                                        Action::None
                                    }
                                    data::FileSystemItem::Note(_) => Action::EditNote,
                                }
                            } else {
                                Action::None
                            }
                        } else {
                            Action::None
                        }
                    } else {
                        Action::EditNote
                    }
                }
                KeyCode::F(12) => Action::ToggleLogs,
                _ => Action::None,
            },
            InputMode::Editing | InputMode::CreatingFolder | InputMode::Renaming => {
                match key.code {
                    KeyCode::Enter => Action::SubmitInput,
                    KeyCode::Esc => Action::CancelInput,
                    KeyCode::Backspace => Action::Backspace,
                    KeyCode::Char(c) => Action::EnterChar(c),
                    _ => Action::None,
                }
            }
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
                    self.status_msg = String::from(" Press 'h' for help ");
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
            InputMode::TagSearch => match key.code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    self.status_msg = String::from("Tag filter active. Esc to clear.");
                    Action::None
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.search_query.clear();
                    self.update_tag_search();
                    self.status_msg = String::from(" Press 'h' for help ");
                    Action::None
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.update_tag_search();
                    self.status_msg = format!("Tag Search: {}", self.search_query);
                    Action::None
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.update_tag_search();
                    self.status_msg = format!("Tag Search: {}", self.search_query);
                    Action::None
                }
                _ => Action::None,
            },
            InputMode::ContentSearch => match key.code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    self.status_msg = String::from("Content filter active. Esc to clear.");
                    Action::None
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.search_query.clear();
                    self.update_search();
                    self.status_msg = String::from(" Press 'h' for help ");
                    Action::None
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.update_content_search();
                    self.status_msg = format!("Content Search: {}", self.search_query);
                    Action::None
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.update_content_search();
                    self.status_msg = format!("Content Search: {}", self.search_query);
                    Action::None
                }
                _ => Action::None,
            },
            InputMode::Help => match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') => {
                    self.input_mode = InputMode::Normal;
                    self.status_msg = String::from(" Press 'h' for help ");
                    Action::None
                }
                _ => Action::None,
            },
        }
    }
}
