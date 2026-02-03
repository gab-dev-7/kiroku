use kiroku_tui::data;
use kiroku_tui::ops;
use tempfile::tempdir;

#[test]
fn test_folder_support() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Test create_note with folder
    let note_path = ops::create_note(root, "work/project_a").unwrap();
    assert!(note_path.exists());
    assert!(root.join("work").exists());
    assert!(root.join("work/project_a.md").exists());

    // 2. Test load_notes
    let path_str = root.to_string_lossy().to_string();
    let notes = data::load_notes(&path_str).unwrap();

    // Find the note we just created
    let note = notes
        .iter()
        .find(|n| n.path == note_path)
        .expect("Note not found");

    // Check title (should be relative path without extension)
    #[cfg(unix)]
    assert_eq!(note.title, "work/project_a");

    // 3. Test deep nesting
    let deep_note_path = ops::create_note(root, "a/b/c/deep").unwrap();
    assert!(deep_note_path.exists());
    assert!(root.join("a/b/c/deep.md").exists());

    let notes_v2 = data::load_notes(&path_str).unwrap();
    let deep_note = notes_v2
        .iter()
        .find(|n| n.path == deep_note_path)
        .expect("Deep note not found");

    #[cfg(unix)]
    assert_eq!(deep_note.title, "a/b/c/deep");

    // 4. Rename into folder
    let new_path = ops::rename_note(&note_path, "subdir/project_b").unwrap();
    assert!(new_path.exists());
    assert!(!note_path.exists());
    assert!(root.join("work/subdir/project_b.md").exists());

    // 5. Test rename with '..' to move up
    let moved_path = ops::rename_note(&new_path, "../../moved").unwrap();
    assert!(root.join("moved.md").exists());
    assert!(moved_path.exists());

    // 6. Test create_folder
    let folder_path = ops::create_folder(root, "new_folder").unwrap();
    assert!(folder_path.exists());
    assert!(folder_path.is_dir());
}
