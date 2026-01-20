use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Note {
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub last_modified: SystemTime,
}

impl Note {
    fn from_path(path: PathBuf) -> Option<Self> {
        let content = fs::read_to_string(&path).ok()?;
        let metadata = fs::metadata(&path).ok()?;

        let title = path.file_stem()?.to_string_lossy().to_string();

        Some(Self {
            path,
            title,
            content,
            last_modified: metadata.modified().unwrap_or(SystemTime::now()),
        })
    }
}

pub fn load_notes(directory: &str) -> Vec<Note> {
    let mut notes = Vec::new();

    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(note) = Note::from_path(path.to_path_buf()) {
                notes.push(note);
            }
        }
    }

    notes.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    notes
}
