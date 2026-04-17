use std::fmt;

#[derive(Debug)]
pub enum JiraError {
    Config(String),
    Api { status: u16, body: String },
    Http(String),
    Json(String),
    Io(std::io::Error),
}

impl fmt::Display for JiraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JiraError::Config(msg) => write!(f, "Config error: {}", msg),
            JiraError::Api { status, body } => write!(f, "API error {}: {}", status, body),
            JiraError::Http(msg) => write!(f, "HTTP error: {}", msg),
            JiraError::Json(msg) => write!(f, "JSON error: {}", msg),
            JiraError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl From<std::io::Error> for JiraError {
    fn from(err: std::io::Error) -> Self {
        JiraError::Io(err)
    }
}

impl From<serde_json::Error> for JiraError {
    fn from(err: serde_json::Error) -> Self {
        JiraError::Json(err.to_string())
    }
}

impl From<serde_yaml::Error> for JiraError {
    fn from(err: serde_yaml::Error) -> Self {
        JiraError::Config(err.to_string())
    }
}

impl From<ureq::Error> for JiraError {
    fn from(err: ureq::Error) -> Self {
        match err {
            ureq::Error::Status(code, resp) => {
                let body = resp.into_string().unwrap_or_default();
                JiraError::Api {
                    status: code,
                    body,
                }
            }
            ureq::Error::Transport(t) => JiraError::Http(t.to_string()),
        }
    }
}
