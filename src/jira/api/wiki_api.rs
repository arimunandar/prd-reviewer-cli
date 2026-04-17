use crate::jira::client::Client;
use crate::jira::error::JiraError;
use crate::jira::models::wiki::*;

pub fn get_page(client: &Client, id: &str, expand: &str) -> Result<WikiPage, JiraError> {
    let mut url = format!("{}/{}", client.base_api, id);
    if !expand.is_empty() {
        url.push_str(&format!("?expand={}", expand));
    }
    client.get(&url)
}

pub fn search_pages(
    client: &Client,
    title: &str,
    cql: &str,
    space: &str,
    max_results: i32,
) -> Result<WikiSearchResult, JiraError> {
    let mut params = Vec::new();

    if !cql.is_empty() {
        params.push(format!("cql={}", url_encode(cql)));
    } else {
        let mut cql_parts = Vec::new();
        if !title.is_empty() {
            cql_parts.push(format!("title~\"{}\"", title));
        }
        if !space.is_empty() {
            cql_parts.push(format!("space=\"{}\"", space));
        }
        if !cql_parts.is_empty() {
            params.push(format!("cql={}", url_encode(&cql_parts.join(" AND "))));
        }
    }
    params.push(format!("limit={}", max_results));

    let url = format!("{}/search?{}", client.base_api, params.join("&"));
    client.get(&url)
}

pub fn create_page(client: &Client, payload: &WikiCreatePage) -> Result<WikiPage, JiraError> {
    client.post(&client.base_api, payload)
}

pub fn update_page(
    client: &Client,
    id: &str,
    payload: &WikiUpdatePage,
) -> Result<WikiPage, JiraError> {
    let url = format!("{}/{}", client.base_api, id);
    client.post(&url, payload) // Wiki uses POST for update via the client.Put equivalent
}

pub fn export_page(
    client: &Client,
    id: &str,
    format: &str,
    output_path: &str,
) -> Result<(), JiraError> {
    let url = format!("{}/{}/export/{}", client.base_api, id, format);
    let data = match client.get_raw(&url) {
        Ok(d) => d,
        Err(_) if format == "pdf" => {
            // Fallback for PDF export plugin
            let base = client.base_wiki.trim_end_matches('/');
            let fallback_url =
                format!("{}/spacesexportpdf/exportpdf.action?pageId={}", base, id);
            client.get_raw(&fallback_url)?
        }
        Err(e) => return Err(e),
    };

    let out = if output_path.is_empty() {
        format!("page_{}.{}", id, format)
    } else {
        output_path.to_string()
    };

    std::fs::write(&out, &data)?;
    println!("Exported to {} ({} bytes)", out, data.len());
    Ok(())
}

/// Add a comment to a wiki page.
pub fn add_comment(
    client: &Client,
    page_id: &str,
    body_html: &str,
) -> Result<(), JiraError> {
    let payload = serde_json::json!({
        "type": "comment",
        "container": {
            "id": page_id,
            "type": "page"
        },
        "body": {
            "storage": {
                "value": body_html,
                "representation": "storage"
            }
        }
    });
    let _: serde_json::Value = client.post(&client.base_api, &payload)?;
    Ok(())
}

pub fn list_spaces(client: &Client) -> Result<Vec<WikiSpace>, JiraError> {
    let base = client.base_wiki.trim_end_matches('/');
    let url = format!("{}/rest/api/space", base);
    let result: WikiSpaceList = client.get(&url)?;
    Ok(result.results)
}

fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
