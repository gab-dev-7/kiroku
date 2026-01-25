use crate::errors::KirokuError;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

// open the user's preferred editor for the given file
pub fn open_editor(
    base_path: &Path,
    file_path: Option<&PathBuf>,
    editor_cmd: Option<&str>,
) -> Result<(), KirokuError> {
    // temporarily disable raw mode to allow the editor to take over the terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    let editor = if let Some(cmd) = editor_cmd {
        cmd.to_string()
    } else {
        std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string())
    };

    let mut cmd = Command::new(editor);
    cmd.current_dir(base_path);

    if let Some(path) = file_path {
        cmd.arg(path);
    }

    let status = cmd.status().map_err(KirokuError::Io)?;

    // restore raw mode after editor exits
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    if !status.success() {
        return Err(KirokuError::Env(
            "Editor exited with non-zero status".into(),
        ));
    }

    Ok(())
}

// create a new markdown file with the given filename
pub fn create_note(base_path: &Path, filename: &str) -> Result<PathBuf, KirokuError> {
    let mut safe_filename = filename.trim().replace(" ", "_");
    if !safe_filename.ends_with(".md") {
        safe_filename.push_str(".md");
    }

    let path = base_path.join(&safe_filename);

    if path.exists() {
        return Err(KirokuError::Io(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "File already exists",
        )));
    }

    fs::File::create(&path)?;
    Ok(path)
}

// permanently delete the specified note file
pub fn delete_note(path: &Path) -> Result<(), KirokuError> {
    fs::remove_file(path)?;
    Ok(())
}

// sync changes with the remote git repository
pub fn run_git_sync(base_path: &Path) -> Result<String, KirokuError> {
    println!("Executing git sync in: {:?}", base_path);
    if !base_path.join(".git").exists() {
        return Err(KirokuError::Git(
            "not a git repo (run 'git init' in folder)".to_string(),
        ));
    }

    // stage all changes
    let add = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(base_path)
        .status()?;

    if !add.success() {
        return Err(KirokuError::Git("git add failed".to_string()));
    }

    // commit changes with a default message
    let _commit = Command::new("git")
        .args(["commit", "-m", "auto-sync from kiroku"])
        .current_dir(base_path)
        .status()?;

    // push changes to remote, allowing interactive auth if needed
    let push = Command::new("git")
        .arg("push")
        .current_dir(base_path)
        .status()?;

    if push.success() {
        Ok("synced!".to_string())
    } else {
        Err(KirokuError::Git("push failed".to_string()))
    }
}
