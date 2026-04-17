use crate::figma::client::Client;
use crate::figma::error::FigmaError;
use crate::figma::models::component::{FileComponentsResponse, FileStylesResponse};
use crate::figma::models::component::{ComponentMeta, StyleMeta};
use crate::figma::models::file::{FileNodesResponse, FileResponse};

pub fn get_file(client: &Client, file_key: &str, depth: i32) -> Result<FileResponse, FigmaError> {
    let url = format!("{}/v1/files/{}?depth={}", client.base_url, file_key, depth);
    client.get(&url)
}

pub fn get_file_nodes(
    client: &Client,
    file_key: &str,
    node_ids: &[&str],
) -> Result<FileNodesResponse, FigmaError> {
    let escaped: Vec<String> = node_ids.iter().map(|id| url_encode(id)).collect();
    let url = format!(
        "{}/v1/files/{}/nodes?ids={}",
        client.base_url,
        file_key,
        escaped.join(",")
    );
    client.get(&url)
}

pub fn get_file_components(
    client: &Client,
    file_key: &str,
) -> Result<Vec<ComponentMeta>, FigmaError> {
    let url = format!("{}/v1/files/{}/components", client.base_url, file_key);
    let result: FileComponentsResponse = client.get(&url)?;
    Ok(result.meta.components)
}

pub fn get_file_styles(client: &Client, file_key: &str) -> Result<Vec<StyleMeta>, FigmaError> {
    let url = format!("{}/v1/files/{}/styles", client.base_url, file_key);
    let result: FileStylesResponse = client.get(&url)?;
    Ok(result.meta.styles)
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
