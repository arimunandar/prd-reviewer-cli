use crate::config::TuntunConfig;
use crate::figma::error::FigmaError;

/// Figma config — reads from ~/.prd-reviewer.yaml figma section
pub struct Config {
    pub personal_token: String,
    pub default_team_id: String,
    #[allow(dead_code)]
    pub export_dir: String,
}

impl Config {
    pub fn load() -> Result<Config, FigmaError> {
        let cfg = TuntunConfig::load().map_err(|e| FigmaError::Config(e))?;
        Ok(Config {
            personal_token: cfg.figma.personal_token,
            default_team_id: cfg.figma.default_team_id,
            export_dir: if cfg.figma.export_dir.is_empty() {
                "./".to_string()
            } else {
                cfg.figma.export_dir
            },
        })
    }

    pub fn validate(&self) -> Result<(), FigmaError> {
        if self.personal_token.is_empty() {
            return Err(FigmaError::Config(
                "figma.personal_token is required. Run install.sh to set up".to_string(),
            ));
        }
        Ok(())
    }
}
