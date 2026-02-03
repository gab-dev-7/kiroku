use anyhow::{Context, Result};
use log::warn;
use serde::Deserialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct Frontmatter {
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileSystemItem {
    Note(Note),
    Folder(PathBuf),
}

// represents a markdown note
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub path: PathBuf,
    pub title: String,
    pub content: Option<String>,
    pub last_modified: SystemTime,
    pub size: u64,
    pub tags: Vec<String>,
}

impl Note {
    // create note from path
    pub fn from_path(path: PathBuf, root: &std::path::Path) -> Result<Self> {
        let metadata = fs::metadata(&path)
            .with_context(|| format!("Failed to get metadata for: {:?}", path))?;

        let relative_path = path.strip_prefix(root).unwrap_or(&path);
        let title = relative_path
            .with_extension("")
            .to_string_lossy()
            .to_string();

        let tags = extract_tags(&path).unwrap_or_default();

        Ok(Self {
            path,
            title,
            content: None,
            last_modified: metadata.modified().unwrap_or(SystemTime::now()),
            size: metadata.len(),
            tags,
        })
    }
}

fn extract_tags(path: &PathBuf) -> Result<Vec<String>> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    // check first line
    if reader.read_line(&mut line)? == 0 || line.trim() != "---" {
        return Ok(Vec::new());
    }

    let mut frontmatter_content = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if line.trim() == "---" {
            break;
        }
        frontmatter_content.push_str(&line);
    }

    // parse yaml frontmatter
    // If it fails, we just assume no valid tags were found in that block
    let fm: Frontmatter =
        serde_yaml::from_str(&frontmatter_content).unwrap_or(Frontmatter { tags: Vec::new() });
    Ok(fm.tags)
}

// read note content
pub fn read_note_content(path: &PathBuf) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))
}

// scan directory for notes
pub fn load_notes(directory: &str) -> Result<Vec<Note>> {
    let mut notes = Vec::new();
    let root = PathBuf::from(directory);

    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            match Note::from_path(path.to_path_buf(), &root) {
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

// scan directory for all items
pub fn load_all_items(directory: &str) -> Result<Vec<FileSystemItem>> {
    let mut items = Vec::new();
    let root = PathBuf::from(directory);

    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // skip root directory
        if path == root {
            continue;
        }

        // skip hidden files
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        if path.is_dir() {
            // keep relative path
            items.push(FileSystemItem::Folder(path.to_path_buf()));
        } else if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("md")
            && let Ok(note) = Note::from_path(path.to_path_buf(), &root)
        {
            items.push(FileSystemItem::Note(note));
        }
    }

    // sort folders then files
    items.sort_by(|a, b| match (a, b) {
        (FileSystemItem::Folder(pa), FileSystemItem::Folder(pb)) => pa.cmp(pb),
        (FileSystemItem::Folder(_), FileSystemItem::Note(_)) => std::cmp::Ordering::Less,
        (FileSystemItem::Note(_), FileSystemItem::Folder(_)) => std::cmp::Ordering::Greater,
        (FileSystemItem::Note(na), FileSystemItem::Note(nb)) => na.title.cmp(&nb.title),
    });

    Ok(items)
}
