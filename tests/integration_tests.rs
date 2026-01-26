use kiroku_tui::app::{App, SortMode};
use kiroku_tui::config::Config;
use kiroku_tui::data::{self, Note};
use kiroku_tui::ops::{create_note, rename_note};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use tempfile::TempDir;

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
}

#[test]
fn test_sorting_logic() {
    let mut n1 = create_test_note("C_last");
    n1.last_modified = SystemTime::now();
    n1.size = 10;

    let mut n2 = create_test_note("A_first");
    n2.last_modified = SystemTime::now() - std::time::Duration::from_secs(100);
    n2.size = 100;

    let mut n3 = create_test_note("B_middle");
    n3.last_modified = SystemTime::now() - std::time::Duration::from_secs(50);
    n3.size = 50;

    let notes = vec![n1.clone(), n2.clone(), n3.clone()];
    let mut app = App::new(notes, PathBuf::from("/tmp"), Config::default());

    // Default is Date (Descending)
    app.sort_mode = SortMode::Date;
    app.sort_notes();
    assert_eq!(app.notes[0].title, "C_last");

    // Name (Ascending)
    app.sort_mode = SortMode::Name;
    app.sort_notes();
    assert_eq!(app.notes[0].title, "A_first");

    // Size (Descending)
    app.sort_mode = SortMode::Size;
    app.sort_notes();
    assert_eq!(app.notes[0].title, "A_first"); // Size 100
}

#[test]
fn test_lru_cache_eviction() {
    let notes: Vec<Note> = (0..15)
        .map(|i| {
            let mut n = create_test_note(&format!("note_{}", i));
            n.content = Some(format!("content_{}", i));
            n
        })
        .collect();

    let mut app = App::new(notes, PathBuf::from("/tmp"), Config::default());

    app.list_state.select(Some(10));

    for i in 1..=10 {
        app.load_note_content(i);
    }

    assert_eq!(app.recent_indices.len(), 10);
    assert!(!app.recent_indices.contains(&0));
    assert!(app.notes[0].content.is_none());
}

#[test]
fn test_note_from_path() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test_note.md");

    let mut file = fs::File::create(&file_path)?;
    use std::io::Write;
    writeln!(file, "# Test Content")?;

    let note = Note::from_path(file_path.clone())?;

    assert_eq!(note.title, "test_note");
    assert_eq!(note.path, file_path);
    assert!(note.size > 0);

    let content = data::read_note_content(&note.path)?;
    assert_eq!(content, "# Test Content\n");

    Ok(())
}

#[test]
fn test_create_note_safe_filename() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let path = create_note(temp.path(), "My Note")?;
    assert!(path.exists());
    assert!(path.to_str().unwrap().contains("My_Note.md"));
    Ok(())
}

#[test]
fn test_rename_note() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let old_path = temp.path().join("old.md");
    fs::File::create(&old_path)?;

    let new_path = rename_note(&old_path, "new name")?;
    assert!(!old_path.exists());
    assert!(new_path.exists());
    assert!(new_path.to_str().unwrap().contains("new_name.md"));

    Ok(())
}

#[test]
fn test_load_notes() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path();

    fs::File::create(dir_path.join("a.md"))?;
    fs::File::create(dir_path.join("b.md"))?;
    fs::File::create(dir_path.join("ignore_me.txt"))?;

    let notes = data::load_notes(dir_path.to_str().unwrap())?;

    assert_eq!(notes.len(), 2);
    assert!(notes.iter().any(|n| n.title == "a"));
    assert!(notes.iter().any(|n| n.title == "b"));

    Ok(())
}
