use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadingPosition {
    pub work_id: String,
    pub section_id: String,
    pub section_index: usize,
    pub scroll: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bookmark {
    pub work_id: String,
    pub section_id: String,
    pub section_title: String,
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: usize,
    pub last_position: Option<ReadingPosition>,
    pub bookmarks: Vec<Bookmark>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: 0,
            last_position: None,
            bookmarks: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct ConfigStore {
    path: PathBuf,
    pub settings: AppConfig,
}

impl ConfigStore {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        let settings = if path.exists() {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read config file {}", path.display()))?;
            serde_json::from_str(&raw)
                .with_context(|| format!("failed to parse config file {}", path.display()))?
        } else {
            AppConfig::default()
        };

        Ok(Self { path, settings })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let serialized = serde_json::to_string_pretty(&self.settings)?;
        fs::write(&self.path, serialized)
            .with_context(|| format!("failed to write config file {}", self.path.display()))
    }

    pub fn as_pretty_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.settings).map_err(Into::into)
    }
}

fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("org", "EMARX", "emarx")
        .context("could not determine a platform config directory")?;
    Ok(dirs.config_dir().join("config.json"))
}
