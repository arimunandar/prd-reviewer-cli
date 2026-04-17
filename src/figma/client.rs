use crate::figma::config::Config;
use crate::figma::error::FigmaError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Read;
use std::time::Duration;

const BASE_URL: &str = "https://api.figma.com";
const MAX_RETRY_DELAY: u64 = 120;
const MAX_RETRIES: u32 = 3;

pub struct Client {
    agent: ureq::Agent,
    pub base_url: String,
    token: String,
}

impl Client {
    pub fn new(cfg: &Config) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(60))
            .timeout_write(Duration::from_secs(30))
            .build();
        Client {
            agent,
            base_url: BASE_URL.to_string(),
            token: cfg.personal_token.clone(),
        }
    }

    pub fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, FigmaError> {
        for attempt in 0..=MAX_RETRIES {
            let resp = self
                .agent
                .get(url)
                .set("X-Figma-Token", &self.token)
                .set("Content-Type", "application/json")
                .call();

            match resp {
                Ok(resp) => {
                    let result: T = resp.into_json().map_err(|e| FigmaError::Json(e.to_string()))?;
                    return Ok(result);
                }
                Err(ureq::Error::Status(429, resp)) => {
                    if attempt == MAX_RETRIES {
                        return Err(FigmaError::Http(format!(
                            "rate limited after {} retries",
                            MAX_RETRIES
                        )));
                    }
                    let delay = parse_retry_after(
                        resp.header("Retry-After").unwrap_or(""),
                    );
                    eprintln!(
                        "Rate limited. Retrying in {} seconds (attempt {}/{})...",
                        delay,
                        attempt + 1,
                        MAX_RETRIES
                    );
                    std::thread::sleep(Duration::from_secs(delay));
                }
                Err(e) => return Err(FigmaError::from(e)),
            }
        }
        Err(FigmaError::Http(format!(
            "rate limited after {} retries",
            MAX_RETRIES
        )))
    }

    #[allow(dead_code)]
    pub fn get_raw(&self, url: &str) -> Result<Vec<u8>, FigmaError> {
        let resp = self
            .agent
            .get(url)
            .set("X-Figma-Token", &self.token)
            .call()?;
        let mut bytes = Vec::new();
        resp.into_reader().read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    pub fn post<P: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        payload: &P,
    ) -> Result<T, FigmaError> {
        let resp = self
            .agent
            .post(url)
            .set("X-Figma-Token", &self.token)
            .set("Content-Type", "application/json")
            .send_json(serde_json::to_value(payload)?)?;
        let result: T = resp.into_json().map_err(|e| FigmaError::Json(e.to_string()))?;
        Ok(result)
    }
}

pub fn download_image(image_url: &str, output_path: &str) -> Result<(), FigmaError> {
    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(60))
        .timeout_write(Duration::from_secs(30))
        .build();
    let resp = agent.get(image_url).call()?;
    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(output_path)?;
    std::io::copy(&mut reader, &mut file)?;
    Ok(())
}

fn parse_retry_after(header: &str) -> u64 {
    if header.is_empty() {
        return 5;
    }
    if let Ok(v) = header.parse::<u64>() {
        if v > MAX_RETRY_DELAY {
            return MAX_RETRY_DELAY;
        }
        if v > 0 {
            return v;
        }
        return 5;
    }
    5
}
