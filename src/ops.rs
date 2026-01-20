use std::path::PathBuf;
use std::process::Command;
use crate::errors::KirokuError;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

pub fn open_editor(base_path: &PathBuf, file_path: Option<&PathBuf>) -> Result<(), KirokuError> {
    execute!(io::stdout(), LeaveAlternateScreen)?;

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let mut cmd = Command::new(editor);
    cmd.current_dir(base_path);

    if let Some(path) = file_path {
        cmd.arg(path);
    }

    // handle error if editor fails to start
    let status = cmd.status().map_err(|e| KirokuError::Io(e))?;

    execute!(io::stdout(), EnterAlternateScreen)?;
    
    if !status.success() {
        return Err(KirokuError::Env("Editor exited with non-zero status".into()));
    }
    
    Ok(())
}

pub fn run_git_sync(base_path: &PathBuf) -> Result<String, KirokuError> {
    if !base_path.join(".git").exists() {
        return Err(KirokuError::Git("not a git repo (run 'git init' in folder)".to_string()));
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
