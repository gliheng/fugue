use crate::error::{FugueError, Result};
use crate::package::PackageManager;
use serde::Deserialize;
use std::path::Path;

const CONFIG_FILENAME: &str = "fugue.toml";

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub framework: Option<String>,
    pub assets_dir: String,
    pub assets_prefix: String,
    pub build_output_dir: String,
    pub server_entry: Option<String>,
    pub install_command: Option<String>,
    pub build_command: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawProjectConfig {
    framework: Option<String>,
    assets: Option<RawAssetsConfig>,
    build: Option<RawBuildConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawAssetsConfig {
    dir: Option<String>,
    prefix: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawBuildConfig {
    output_dir: Option<String>,
    server_entry: Option<String>,
    install: Option<String>,
    command: Option<String>,
}

impl ProjectConfig {
    pub fn load(project_dir: &Path) -> Result<Self> {
        let config_path = project_dir.join(CONFIG_FILENAME);

        if !config_path.exists() {
            return Err(FugueError::ConfigError(format!(
                "{} not found. Create one and set `framework` to one of: worker, nuxtjs, react-router, vite, hono",
                CONFIG_FILENAME
            )));
        }

        let content = std::fs::read_to_string(&config_path).map_err(|e| {
            FugueError::ConfigError(format!("Failed to read {}: {}", CONFIG_FILENAME, e))
        })?;

        let raw: RawProjectConfig = toml::from_str(&content).map_err(|e| {
            FugueError::ConfigError(format!("Failed to parse {}: {}", CONFIG_FILENAME, e))
        })?;

        let framework = raw.framework.as_deref().ok_or_else(|| {
            FugueError::ConfigError(format!(
                "`framework` is required in {} (e.g., framework = \"hono\")",
                CONFIG_FILENAME
            ))
        })?;

        let defaults = Self::for_framework(framework);
        Ok(defaults.merge(raw))
    }

    /// Returns the install command string.
    /// If `build.install` is set in `fugue.toml`, uses that.
    /// Otherwise falls back to package-manager defaults.
    pub fn get_install_command(&self, pm: &PackageManager) -> String {
        if let Some(cmd) = &self.install_command {
            cmd.clone()
        } else {
            match pm {
                PackageManager::Npm => "npm install".to_string(),
                PackageManager::Yarn => "yarn install".to_string(),
                PackageManager::Pnpm => "pnpm install".to_string(),
            }
        }
    }

    /// Returns the build command string.
    /// If `build.command` is set in `fugue.toml`, uses that.
    /// Otherwise falls back to package-manager defaults.
    pub fn get_build_command(&self, pm: &PackageManager) -> String {
        if let Some(cmd) = &self.build_command {
            cmd.clone()
        } else {
            match pm {
                PackageManager::Npm => "npm run build".to_string(),
                PackageManager::Yarn => "yarn build".to_string(),
                PackageManager::Pnpm => "pnpm build".to_string(),
            }
        }
    }

    pub fn for_framework(framework: &str) -> Self {
        let framework = Some(framework.to_string());
        match framework.as_deref().unwrap_or("worker") {
            "nuxtjs" => Self {
                framework,
                assets_dir: ".output/public".to_string(),
                assets_prefix: "/_nuxt/".to_string(),
                build_output_dir: ".output".to_string(),
                server_entry: Some("server/index.mjs".to_string()),
                install_command: None,
                build_command: None,
            },
            "react-router" => Self {
                framework,
                assets_dir: "build/client".to_string(),
                assets_prefix: String::new(),
                build_output_dir: "build".to_string(),
                server_entry: Some("server/index.js".to_string()),
                install_command: None,
                build_command: None,
            },
            "vite" => Self {
                framework,
                assets_dir: "dist/client".to_string(),
                assets_prefix: String::new(),
                build_output_dir: "dist".to_string(),
                server_entry: Some("vite_app/index.js".to_string()),
                install_command: None,
                build_command: None,
            },
            "hono" => Self {
                framework,
                assets_dir: "public".to_string(),
                assets_prefix: String::new(),
                build_output_dir: ".".to_string(),
                server_entry: None,
                install_command: None,
                build_command: None,
            },
            _ => Self {
                framework,
                assets_dir: "public".to_string(),
                assets_prefix: String::new(),
                build_output_dir: ".".to_string(),
                server_entry: None,
                install_command: None,
                build_command: None,
            },
        }
    }

    fn merge(self, raw: RawProjectConfig) -> Self {
        let assets_prefix = raw
            .assets
            .as_ref()
            .and_then(|a| a.prefix.clone())
            .unwrap_or(self.assets_prefix);

        Self {
            framework: raw.framework.or(self.framework),
            assets_dir: raw
                .assets
                .as_ref()
                .and_then(|a| a.dir.clone())
                .unwrap_or(self.assets_dir),
            assets_prefix,
            build_output_dir: raw
                .build
                .as_ref()
                .and_then(|b| b.output_dir.clone())
                .unwrap_or(self.build_output_dir),
            server_entry: raw
                .build
                .as_ref()
                .and_then(|b| b.server_entry.clone())
                .or(self.server_entry),
            install_command: raw
                .build
                .as_ref()
                .and_then(|b| b.install.clone())
                .or(self.install_command),
            build_command: raw
                .build
                .as_ref()
                .and_then(|b| b.command.clone())
                .or(self.build_command),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_defaults() {
        let cfg = ProjectConfig::for_framework("worker");
        assert_eq!(cfg.framework, Some("worker".to_string()));
        assert_eq!(cfg.assets_dir, "public");
        assert_eq!(cfg.assets_prefix, "");
        assert_eq!(cfg.build_output_dir, ".");
        assert_eq!(cfg.server_entry, None);
        assert_eq!(cfg.install_command, None);
        assert_eq!(cfg.build_command, None);
    }

    #[test]
    fn test_nuxtjs_defaults() {
        let cfg = ProjectConfig::for_framework("nuxtjs");
        assert_eq!(cfg.framework, Some("nuxtjs".to_string()));
        assert_eq!(cfg.assets_dir, ".output/public");
        assert_eq!(cfg.assets_prefix, "/_nuxt/");
        assert_eq!(cfg.build_output_dir, ".output");
        assert_eq!(cfg.server_entry, Some("server/index.mjs".to_string()));
        assert_eq!(cfg.install_command, None);
        assert_eq!(cfg.build_command, None);
    }

    #[test]
    fn test_reactrouter_defaults() {
        let cfg = ProjectConfig::for_framework("react-router");
        assert_eq!(cfg.framework, Some("react-router".to_string()));
        assert_eq!(cfg.assets_dir, "build/client");
        assert_eq!(cfg.assets_prefix, "");
        assert_eq!(cfg.build_output_dir, "build");
        assert_eq!(cfg.server_entry, Some("server/index.js".to_string()));
        assert_eq!(cfg.install_command, None);
        assert_eq!(cfg.build_command, None);
    }

    #[test]
    fn test_vite_defaults() {
        let cfg = ProjectConfig::for_framework("vite");
        assert_eq!(cfg.framework, Some("vite".to_string()));
        assert_eq!(cfg.assets_dir, "dist/client");
        assert_eq!(cfg.assets_prefix, "");
        assert_eq!(cfg.build_output_dir, "dist");
        assert_eq!(cfg.server_entry, Some("vite_app/index.js".to_string()));
        assert_eq!(cfg.install_command, None);
        assert_eq!(cfg.build_command, None);
    }

    #[test]
    fn test_hono_defaults() {
        let cfg = ProjectConfig::for_framework("hono");
        assert_eq!(cfg.framework, Some("hono".to_string()));
        assert_eq!(cfg.assets_dir, "public");
        assert_eq!(cfg.assets_prefix, "");
        assert_eq!(cfg.build_output_dir, ".");
        assert_eq!(cfg.server_entry, None);
        assert_eq!(cfg.install_command, None);
        assert_eq!(cfg.build_command, None);
    }

    #[test]
    fn test_unknown_framework_uses_worker_defaults() {
        let cfg = ProjectConfig::for_framework("unknown");
        assert_eq!(cfg.framework, Some("unknown".to_string()));
        assert_eq!(cfg.assets_dir, "public");
        assert_eq!(cfg.assets_prefix, "");
        assert_eq!(cfg.build_output_dir, ".");
        assert_eq!(cfg.server_entry, None);
        assert_eq!(cfg.install_command, None);
        assert_eq!(cfg.build_command, None);
    }

    #[test]
    fn test_merge_partial_override() {
        let defaults = ProjectConfig::for_framework("nuxtjs");
        let raw = RawProjectConfig {
            framework: None,
            assets: Some(RawAssetsConfig {
                dir: Some("dist/static".to_string()),
                prefix: None,
            }),
            build: None,
        };
        let cfg = defaults.merge(raw);
        assert_eq!(cfg.assets_dir, "dist/static");
        assert_eq!(cfg.assets_prefix, "/_nuxt/");
        assert_eq!(cfg.build_output_dir, ".output");
        assert_eq!(cfg.server_entry, Some("server/index.mjs".to_string()));
    }

    #[test]
    fn test_merge_empty_raw() {
        let defaults = ProjectConfig::for_framework("react-router");
        let raw = RawProjectConfig::default();
        let cfg = defaults.merge(raw);
        assert_eq!(cfg.assets_dir, "build/client");
        assert_eq!(cfg.assets_prefix, "");
        assert_eq!(cfg.build_output_dir, "build");
        assert_eq!(cfg.server_entry, Some("server/index.js".to_string()));
    }

    #[test]
    fn test_load_from_dir_no_file_fails() {
        let dir = std::env::temp_dir().join("fugue-test-no-config");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        assert!(ProjectConfig::load(&dir).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_from_dir_with_file() {
        let dir = std::env::temp_dir().join("fugue-test-with-config");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(
            dir.join("fugue.toml"),
            "framework = \"nuxtjs\"\n[assets]\ndir = \"static\"\nprefix = \"/assets/\"\n",
        )
        .unwrap();

        let cfg = ProjectConfig::load(&dir).unwrap();
        assert_eq!(cfg.assets_dir, "static");
        assert_eq!(cfg.assets_prefix, "/assets/");
        assert_eq!(cfg.build_output_dir, ".output");
        assert_eq!(cfg.server_entry, Some("server/index.mjs".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_from_dir_with_partial_file() {
        let dir = std::env::temp_dir().join("fugue-test-partial-config");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(
            dir.join("fugue.toml"),
            "framework = \"react-router\"\n[build]\noutput_dir = \"dist\"\n",
        )
        .unwrap();

        let cfg = ProjectConfig::load(&dir).unwrap();
        assert_eq!(cfg.assets_dir, "build/client");
        assert_eq!(cfg.assets_prefix, "");
        assert_eq!(cfg.build_output_dir, "dist");
        assert_eq!(cfg.server_entry, Some("server/index.js".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_build_command_override() {
        let dir = std::env::temp_dir().join("fugue-test-build-cmd");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(
            dir.join("fugue.toml"),
            "framework = \"nuxtjs\"\n[build]\ncommand = \"npm run build:staging\"\n",
        )
        .unwrap();

        let cfg = ProjectConfig::load(&dir).unwrap();
        assert_eq!(cfg.build_command, Some("npm run build:staging".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_build_command_from_config() {
        let mut cfg = ProjectConfig::for_framework("nuxtjs");
        cfg.build_command = Some("pnpm build".to_string());
        let pm = PackageManager::Npm;
        let cmd = cfg.get_build_command(&pm);
        assert_eq!(cmd, "pnpm build");
    }

    #[test]
    fn test_get_build_command_fallback() {
        let cfg = ProjectConfig::for_framework("react-router");
        let pm = PackageManager::Yarn;
        let cmd = cfg.get_build_command(&pm);
        assert_eq!(cmd, "yarn build");
    }

    #[test]
    fn test_load_install_command_override() {
        let dir = std::env::temp_dir().join("fugue-test-install-cmd");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(
            dir.join("fugue.toml"),
            "framework = \"nuxtjs\"\n[build]\ninstall = \"pnpm install --frozen-lockfile\"\n",
        )
        .unwrap();

        let cfg = ProjectConfig::load(&dir).unwrap();
        assert_eq!(
            cfg.install_command,
            Some("pnpm install --frozen-lockfile".to_string())
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_install_command_from_config() {
        let mut cfg = ProjectConfig::for_framework("nuxtjs");
        cfg.install_command = Some("npm ci".to_string());
        let pm = PackageManager::Yarn;
        let cmd = cfg.get_install_command(&pm);
        assert_eq!(cmd, "npm ci");
    }

    #[test]
    fn test_get_install_command_fallback() {
        let cfg = ProjectConfig::for_framework("react-router");
        let pm = PackageManager::Pnpm;
        let cmd = cfg.get_install_command(&pm);
        assert_eq!(cmd, "pnpm install");
    }
}
