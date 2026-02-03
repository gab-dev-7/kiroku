use crate::errors::KirokuError;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

// open user editor
pub fn open_editor(
    base_path: &Path,
    file_path: Option<&PathBuf>,
    editor_cmd: Option<&str>,
) -> Result<(), KirokuError> {
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

    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    if !status.success() {
        return Err(KirokuError::Env(
            "Editor exited with non-zero status".into(),
        ));
    }

    Ok(())
}

// create markdown file
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

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::File::create(&path)?;
    Ok(path)
}

// create directory
pub fn create_folder(base_path: &Path, foldername: &str) -> Result<PathBuf, KirokuError> {
    let safe_foldername = foldername.trim().replace(" ", "_");
    let path = base_path.join(safe_foldername);

    if path.exists() {
        return Err(KirokuError::Io(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Folder already exists",
        )));
    }

    fs::create_dir_all(&path)?;
    Ok(path)
}

// delete note file
pub fn delete_note(path: &Path) -> Result<(), KirokuError> {
    fs::remove_file(path)?;
    Ok(())
}

// rename note
pub fn rename_note(old_path: &Path, new_filename: &str) -> Result<PathBuf, KirokuError> {
    let mut safe_filename = new_filename.trim().replace(" ", "_");
    if !safe_filename.ends_with(".md") {
        safe_filename.push_str(".md");
    }

    let parent = old_path
        .parent()
        .ok_or_else(|| KirokuError::Env("Could not determine parent directory".into()))?;
    let new_path = parent.join(&safe_filename);

    if new_path.exists() {
        return Err(KirokuError::Io(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "File already exists",
        )));
    }

    if let Some(new_parent) = new_path.parent() {
        fs::create_dir_all(new_parent)?;
    }

    fs::rename(old_path, &new_path)?;
    Ok(new_path)
}

// sync with git
pub fn run_git_sync(base_path: &Path) -> Result<String, KirokuError> {
    println!("Executing git sync in: {:?}", base_path);
    if !base_path.join(".git").exists() {
        return Err(KirokuError::Git(
            "not a git repo (run 'git init' in folder)".to_string(),
        ));
    }

    // check for local changes
    let status_out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(base_path)
        .output()?;
    let has_changes = !status_out.stdout.is_empty();

    // check if ahead of remote
    let ahead_out = Command::new("git")
        .args(["rev-list", "HEAD@{u}..HEAD"])
        .current_dir(base_path)
        .output()?;
    let is_ahead = !ahead_out.stdout.is_empty();

    if !has_changes && !is_ahead {
        return Ok("already up to date".to_string());
    }

    if has_changes {
        // stage changes
        let add = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(base_path)
            .status()?;

        if !add.success() {
            return Err(KirokuError::Git("git add failed".to_string()));
        }

        let _commit = Command::new("git")
            .args(["commit", "-m", "auto-sync from kiroku"])
            .current_dir(base_path)
            .status()?;
    }

    // push to remote if needed
    let ahead_after_commit = Command::new("git")
        .args(["rev-list", "@{u}..HEAD"])
        .current_dir(base_path)
        .output()?;

    if !ahead_after_commit.stdout.is_empty() {
        let push = Command::new("git")
            .arg("push")
            .current_dir(base_path)
            .status()?;

        if push.success() {
            Ok("synced!".to_string())
        } else {
            Err(KirokuError::Git("push failed".to_string()))
        }
    } else {
        Ok("synced locally (no remote configured or no push needed)".to_string())
    }
}
