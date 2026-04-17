use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Unified config file at ~/.prd-reviewer.yaml
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TuntunConfig {
    #[serde(default)]
    pub jira: JiraSection,
    #[serde(default)]
    pub wiki: WikiSection,
    #[serde(default)]
    pub figma: FigmaSection,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct JiraSection {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WikiSection {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FigmaSection {
    #[serde(default)]
    pub personal_token: String,
    #[serde(default)]
    pub default_team_id: String,
    #[serde(default = "default_export_dir")]
    pub export_dir: String,
}

fn default_export_dir() -> String {
    "./".to_string()
}

impl TuntunConfig {
    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".prd-reviewer.yaml")
    }

    pub fn load() -> Result<TuntunConfig, String> {
        let p = Self::default_path();
        let data = fs::read_to_string(&p).map_err(|e| {
            format!(
                "failed to read config at {}: {}\nRun install.sh to set up credentials",
                p.display(), e
            )
        })?;
        let cfg: TuntunConfig = serde_yaml::from_str(&data).map_err(|e| e.to_string())?;
        Ok(cfg)
    }

}
