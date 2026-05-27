use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
}

impl PackageManager {
    pub fn detect(project_dir: &Path) -> Self {
        if project_dir.join("pnpm-lock.yaml").exists() {
            PackageManager::Pnpm
        } else if project_dir.join("yarn.lock").exists() {
            PackageManager::Yarn
        } else {
            PackageManager::Npm
        }
    }

    pub fn install_command(&self) -> &str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
        }
    }

    pub fn build_command(&self) -> Vec<&str> {
        match self {
            PackageManager::Npm => vec!["npm", "run", "build"],
            PackageManager::Yarn => vec!["yarn", "build"],
            PackageManager::Pnpm => vec!["pnpm", "build"],
        }
    }
}
