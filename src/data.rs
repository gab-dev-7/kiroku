use anyhow::{Context, Result};
use log::warn;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

// represents a single markdown note file
#[derive(Debug, Clone)]
pub struct Note {
    pub path: PathBuf,
    pub title: String,
    pub content: Option<String>,
    pub last_modified: SystemTime,
    pub size: u64,
}

impl Note {
    // create a note object from a file path
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
            size: metadata.len(),
        })
    }
}

// read the full text content of a note file
pub fn read_note_content(path: &PathBuf) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))
}

// scan directory and return a list of markdown notes sorted by modification date
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_note_from_path() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_note.md");

        let mut file = File::create(&file_path)?;
        writeln!(file, "# Test Content")?;

        let note = Note::from_path(file_path.clone())?;

        assert_eq!(note.title, "test_note");
        assert_eq!(note.path, file_path);
        assert_eq!(note.content, None);
        assert!(note.size > 0);

        // Test reading content
        let content = read_note_content(&note.path)?;
        assert_eq!(content, "# Test Content\n");

        Ok(())
    }

    #[test]
    fn test_load_notes() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();

        File::create(dir_path.join("a.md"))?;
        File::create(dir_path.join("b.md"))?;
        File::create(dir_path.join("ignore_me.txt"))?;

        let notes = load_notes(dir_path.to_str().unwrap())?;

        assert_eq!(notes.len(), 2);
        assert!(notes.iter().any(|n| n.title == "a"));
        assert!(notes.iter().any(|n| n.title == "b"));

        Ok(())
    }
}
