use anyhow::{Context, Result};
use log::warn;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Note {
    pub path: PathBuf,
    pub title: String,
    pub content: Option<String>,
    pub last_modified: SystemTime,
}

impl Note {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let metadata = fs::metadata(&path)
            .with_context(|| format!("Failed to get metadata for: {:?}", path))?;

        let title = path
            .file_stem()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            path,
            title,
            content: None,
            last_modified: metadata.modified().unwrap_or(SystemTime::now()),
        })
    }
}

pub fn read_note_content(path: &PathBuf) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))
}

pub fn load_notes(directory: &str) -> Result<Vec<Note>> {
    let mut notes = Vec::new();

    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            match Note::from_path(path.to_path_buf()) {
                Ok(note) => notes.push(note),
                Err(e) => {
                    warn!("Skipping file {:?}: {}", path, e);
                }
            }
        }
    }

    notes.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    Ok(notes)
}
