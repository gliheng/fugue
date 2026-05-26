use std::path::Path;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".nuxt",
    ".output",
    ".wrangler",
    ".react-router",
    "build",
    ".git",
    ".mf",
    "test",
];

const SKIP_FILES: &[&str] = &[
    "static-assets.mjs",
    "embed-assets.mjs",
    "build-workerd.sh",
    "run-workerd.sh",
    "workerd.capnp",
    "worker-configuration.d.ts",
    "README.md",
    "package-lock.json",
];

fn templates_dir() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let cwd_examples = cwd.join("examples");
    if cwd_examples.join("worker").is_dir() {
        return Some(cwd_examples);
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let exe_examples = parent.join("examples");
            if exe_examples.join("worker").is_dir() {
                return Some(exe_examples);
            }
        }
    }

    None
}

fn should_skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name) || (name.starts_with('.') && name != ".gitignore")
}

fn should_skip_file(name: &str) -> bool {
    if name.starts_with('.') && name != ".gitignore" {
        return true;
    }
    if SKIP_FILES.contains(&name) {
        return true;
    }
    false
}

fn walk_template_dir(
    base: &Path,
    current: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if should_skip_dir(&name) {
                continue;
            }
            walk_template_dir(base, &path, files)?;
        } else if path.is_file() {
            if should_skip_file(&name) {
                continue;
            }
            let relative = path.strip_prefix(base).unwrap_or(&path);
            let key = relative.to_string_lossy().to_string();
            let content = std::fs::read(&path)?;
            files.push((key, content));
        }
    }
    Ok(())
}

pub fn get_template_files(framework: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
    let templates = templates_dir().ok_or_else(|| {
        "Templates directory not found. Expected 'examples/' in current directory or next to binary.".to_string()
    })?;

    match framework {
        "worker" => {
            let dir = templates.join("worker");
            if !dir.is_dir() {
                return Err("Worker template not found at examples/worker/".to_string());
            }
            let mut files = Vec::new();
            walk_template_dir(&dir, &dir, &mut files)
                .map_err(|e| format!("Failed to read worker template: {}", e))?;
            Ok(files)
        }
        "nuxtjs" => {
            let dir = templates.join("nuxtjs-simple");
            if !dir.is_dir() {
                return Err("Nuxt.js template not found at examples/nuxtjs-simple/".to_string());
            }
            let mut files = Vec::new();
            walk_template_dir(&dir, &dir, &mut files)
                .map_err(|e| format!("Failed to read nuxtjs template: {}", e))?;
            Ok(files)
        }
        "react-router" => {
            let dir = templates.join("react-router-simple");
            if !dir.is_dir() {
                return Err(
                    "React Router template not found at examples/react-router-simple/".to_string()
                );
            }
            let mut files = Vec::new();
            walk_template_dir(&dir, &dir, &mut files)
                .map_err(|e| format!("Failed to read react-router template: {}", e))?;
            Ok(files)
        }
        _ => Err(format!("Unknown framework: {}", framework)),
    }
}

#[allow(dead_code)]
pub fn populate_template_source(
    app_id: &uuid::Uuid,
    framework: &str,
) -> Result<std::path::PathBuf, String> {
    let source_dir = crate::config::apps_data_dir()
        .join(app_id.to_string())
        .join("source");
    std::fs::create_dir_all(&source_dir)
        .map_err(|e| format!("Failed to create source directory: {}", e))?;

    let files = get_template_files(framework)?;

    let mut written = 0u64;
    for (relative_path, content) in &files {
        let file_path = source_dir.join(relative_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
        }
        std::fs::write(&file_path, content)
            .map_err(|e| format!("Failed to write {}: {}", relative_path, e))?;
        written += 1;
    }

    tracing::info!(
        "Populated {} template files for '{}' framework in {}",
        written,
        framework,
        source_dir.display()
    );

    Ok(source_dir)
}