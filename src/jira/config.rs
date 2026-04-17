use crate::config::TuntunConfig;
use crate::jira::error::JiraError;

pub struct Config {
    pub jira_token: String,
    pub wiki_token: String,
    pub base_url_jira: String,
    pub base_url_api: String,
    pub base_url_wiki: String,
}

impl Config {
    pub fn load() -> Result<Config, JiraError> {
        let cfg = TuntunConfig::load().map_err(|e| JiraError::Config(e))?;
        let wiki_base = derive_wiki_base(&cfg.wiki.base_url);
        Ok(Config {
            jira_token: cfg.jira.access_token,
            wiki_token: cfg.wiki.access_token,
            base_url_jira: cfg.jira.base_url,
            base_url_api: cfg.wiki.base_url,
            base_url_wiki: wiki_base,
        })
    }

    pub fn validate(&self) -> Result<(), JiraError> {
        if self.base_url_jira.is_empty() {
            return Err(JiraError::Config("jira.base_url is required".to_string()));
        }
        if self.jira_token.is_empty() {
            return Err(JiraError::Config(
                "jira.access_token is required. Run install.sh to set up credentials".to_string(),
            ));
        }
        Ok(())
    }
}

fn derive_wiki_base(api_url: &str) -> String {
    if let Some(idx) = api_url.find("/rest/api/") {
        api_url[..idx].to_string()
    } else {
        api_url.trim_end_matches('/').to_string()
    }
}

