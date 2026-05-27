use crate::error::{FugueError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn calculate_dir_size(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                total_size += calculate_dir_size(&path)?;
            } else {
                total_size += entry.metadata()?.len();
            }
        }
    }

    Ok(total_size)
}

pub fn find_esbuild() -> Result<PathBuf> {
    let local = std::env::current_dir()
        .unwrap_or_default()
        .join("node_modules/.bin/esbuild");
    if local.exists() {
        return Ok(local);
    }

    if let Ok(output) = Command::new("npx")
        .args(["esbuild", "--version"])
        .output()
    {
        if output.status.success() {
            return Ok(PathBuf::from("npx"));
        }
    }

    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            let esbuild = Path::new(dir).join("esbuild");
            if esbuild.exists() {
                return Ok(esbuild);
            }
        }
    }

    Err(FugueError::BuildError(
        "esbuild not found. Install it with: npm install -g esbuild".to_string(),
    ))
}
