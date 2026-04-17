use crate::jira::config::Config;
use crate::jira::error::JiraError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Read;
use std::time::Duration;

pub struct Client {
    agent: ureq::Agent,
    pub base_jira: String,
    pub base_wiki: String,
    pub base_api: String,
    jira_token: String,
    wiki_token: String,
    jira_host: String,
    wiki_host: String,
}

impl Client {
    pub fn new(cfg: &Config, insecure: bool) -> Self {
        let agent = if insecure {
            let tls_config = rustls::ClientConfig::builder_with_provider(
                    std::sync::Arc::new(rustls::crypto::ring::default_provider())
                )
                .with_safe_default_protocol_versions()
                .unwrap()
                .dangerous()
                .with_custom_certificate_verifier(std::sync::Arc::new(NoVerify))
                .with_no_client_auth();

            ureq::AgentBuilder::new()
                .timeout_read(Duration::from_secs(60))
                .timeout_write(Duration::from_secs(30))
                .tls_config(std::sync::Arc::new(tls_config))
                .build()
        } else {
            ureq::AgentBuilder::new()
                .timeout_read(Duration::from_secs(60))
                .timeout_write(Duration::from_secs(30))
                .build()
        };

        let jira_host = extract_host(&cfg.base_url_jira);
        let wiki_host = extract_host(&cfg.base_url_wiki);

        Client {
            agent,
            base_jira: cfg.base_url_jira.clone(),
            base_wiki: cfg.base_url_wiki.clone(),
            base_api: cfg.base_url_api.clone(),
            jira_token: cfg.jira_token.clone(),
            wiki_token: cfg.wiki_token.clone(),
            jira_host,
            wiki_host,
        }
    }

    fn auth_header(&self, url: &str) -> (&str, String) {
        let host = extract_host(url);

        // Route Bearer token by hostname
        if host == self.jira_host && !self.jira_token.is_empty() {
            return ("Authorization", format!("Bearer {}", self.jira_token));
        }
        if host == self.wiki_host && !self.wiki_token.is_empty() {
            return ("Authorization", format!("Bearer {}", self.wiki_token));
        }

        // Fallback: Jira token for unknown hosts
        if !self.jira_token.is_empty() {
            return ("Authorization", format!("Bearer {}", self.jira_token));
        }

        ("", String::new())
    }

    pub fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, JiraError> {
        let (key, val) = self.auth_header(url);
        let mut req = self.agent.get(url).set("Content-Type", "application/json");
        if !key.is_empty() {
            req = req.set(key, &val);
        }
        let resp = req.call()?;
        let result: T = resp.into_json().map_err(|e| JiraError::Json(e.to_string()))?;
        Ok(result)
    }

    pub fn get_raw(&self, url: &str) -> Result<Vec<u8>, JiraError> {
        let (key, val) = self.auth_header(url);
        let mut req = self.agent.get(url);
        if !key.is_empty() {
            req = req.set(key, &val);
        }
        let resp = req.call()?;
        let mut bytes = Vec::new();
        resp.into_reader().read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    pub fn post<P: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        payload: &P,
    ) -> Result<T, JiraError> {
        self.do_json("POST", url, payload)
    }

    pub fn put<P: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        payload: &P,
    ) -> Result<T, JiraError> {
        self.do_json("PUT", url, payload)
    }

    pub fn post_no_response<P: Serialize>(&self, url: &str, payload: &P) -> Result<(), JiraError> {
        let (key, val) = self.auth_header(url);
        let mut req = self.agent.post(url).set("Content-Type", "application/json");
        if !key.is_empty() {
            req = req.set(key, &val);
        }
        req.send_json(serde_json::to_value(payload)?)?;
        Ok(())
    }

    pub fn put_no_response<P: Serialize>(&self, url: &str, payload: &P) -> Result<(), JiraError> {
        let (key, val) = self.auth_header(url);
        let mut req = self.agent.put(url).set("Content-Type", "application/json");
        if !key.is_empty() {
            req = req.set(key, &val);
        }
        req.send_json(serde_json::to_value(payload)?)?;
        Ok(())
    }

    fn do_json<P: Serialize, T: DeserializeOwned>(
        &self,
        method: &str,
        url: &str,
        payload: &P,
    ) -> Result<T, JiraError> {
        let (key, val) = self.auth_header(url);
        let json_val = serde_json::to_value(payload)?;
        let resp = match method {
            "POST" => {
                let mut req = self.agent.post(url).set("Content-Type", "application/json");
                if !key.is_empty() {
                    req = req.set(key, &val);
                }
                req.send_json(json_val)?
            }
            "PUT" => {
                let mut req = self.agent.put(url).set("Content-Type", "application/json");
                if !key.is_empty() {
                    req = req.set(key, &val);
                }
                req.send_json(json_val)?
            }
            _ => return Err(JiraError::Http(format!("unsupported method: {}", method))),
        };
        let result: T = resp.into_json().map_err(|e| JiraError::Json(e.to_string()))?;
        Ok(result)
    }

    /// Get the Agile API base URL (for boards, sprints)
    pub fn agile_base(&self) -> String {
        if let Some(idx) = self.base_jira.find("/rest/api/") {
            format!("{}/rest/agile/1.0", &self.base_jira[..idx])
        } else {
            self.base_jira.clone()
        }
    }

    /// Get the raw ureq agent for image downloading
    pub fn raw_agent(&self) -> &ureq::Agent {
        &self.agent
    }

    /// Get auth header for a URL (for image downloading)
    pub fn auth_for_url(&self, url: &str) -> Option<(String, String)> {
        let (key, val) = self.auth_header(url);
        if key.is_empty() {
            None
        } else {
            Some((key.to_string(), val))
        }
    }
}

fn extract_host(url: &str) -> String {
    if let Some(start) = url.find("://") {
        let rest = &url[start + 3..];
        if let Some(end) = rest.find('/') {
            return rest[..end].to_string();
        }
        return rest.to_string();
    }
    url.to_string()
}

/// TLS certificate verifier that accepts all certificates (for --insecure)
#[derive(Debug)]
struct NoVerify;

impl rustls::client::danger::ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}
