use crate::errors::KirokuError;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

pub fn open_editor(base_path: &PathBuf, file_path: Option<&PathBuf>, editor_cmd: Option<&str>) -> Result<(), KirokuError> {
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

    let status = cmd.status().map_err(|e| KirokuError::Io(e))?;

    execute!(io::stdout(), EnterAlternateScreen)?;

    if !status.success() {
        return Err(KirokuError::Env(
            "Editor exited with non-zero status".into(),
        ));
    }

    Ok(())
}

pub fn create_note(base_path: &PathBuf, filename: &str) -> Result<PathBuf, KirokuError> {
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

pub fn delete_note(path: &PathBuf) -> Result<(), KirokuError> {
    fs::remove_file(path)?;
    Ok(())
}

pub fn run_git_sync(base_path: &PathBuf) -> Result<String, KirokuError> {
    if !base_path.join(".git").exists() {
        return Err(KirokuError::Git(
            "not a git repo (run 'git init' in folder)".to_string(),
        ));
    }

    let add = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(base_path)
        .output()?;

    if !add.status.success() {
        return Err(KirokuError::Git("git add failed".to_string()));
    }

    let _commit = Command::new("git")
        .args(["commit", "-m", "auto-sync from kiroku"])
        .current_dir(base_path)
        .output()?;

    let push = Command::new("git")
        .arg("push")
        .current_dir(base_path)
        .output()?;

    if push.status.success() {
        Ok("synced!".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&push.stderr);
        Err(KirokuError::Git(format!("push failed: {}", stderr)))
    }
}
