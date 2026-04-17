use std::fmt;

#[derive(Debug)]
pub enum FigmaError {
    Config(String),
    Api { status: u16, body: String },
    Parse(String),
    Io(std::io::Error),
    Http(String),
    Json(String),
}

impl fmt::Display for FigmaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FigmaError::Config(msg) => write!(f, "Config error: {}", msg),
            FigmaError::Api { status, body } => write!(f, "API error {}: {}", status, body),
            FigmaError::Parse(msg) => write!(f, "Parse error: {}", msg),
            FigmaError::Io(err) => write!(f, "IO error: {}", err),
            FigmaError::Http(msg) => write!(f, "HTTP error: {}", msg),
            FigmaError::Json(msg) => write!(f, "JSON error: {}", msg),
        }
    }
}

impl From<std::io::Error> for FigmaError {
    fn from(err: std::io::Error) -> Self {
        FigmaError::Io(err)
    }
}

impl From<serde_json::Error> for FigmaError {
    fn from(err: serde_json::Error) -> Self {
        FigmaError::Json(err.to_string())
    }
}

impl From<serde_yaml::Error> for FigmaError {
    fn from(err: serde_yaml::Error) -> Self {
        FigmaError::Config(err.to_string())
    }
}

impl From<ureq::Error> for FigmaError {
    fn from(err: ureq::Error) -> Self {
        match err {
            ureq::Error::Status(code, resp) => {
                let body = resp.into_string().unwrap_or_default();
                FigmaError::Api {
                    status: code,
                    body,
                }
            }
            ureq::Error::Transport(t) => FigmaError::Http(t.to_string()),
        }
    }
}
