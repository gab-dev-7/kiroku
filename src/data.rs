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

// represents a single markdown note file
#[derive(Debug, Clone)]
pub struct Note {
    pub path: PathBuf,
    pub title: String,
    pub content: Option<String>,
    pub last_modified: SystemTime,
    pub size: u64,
    pub tags: Vec<String>,
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

    // Check first line
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

    // Attempt to parse YAML
    // If it fails, we just assume no valid tags were found in that block
    let fm: Frontmatter = serde_yaml::from_str(&frontmatter_content).unwrap_or(Frontmatter {
        tags: Vec::new(),
    });
    Ok(fm.tags)
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
