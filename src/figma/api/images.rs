use crate::figma::client::{self, Client};
use crate::figma::error::FigmaError;
use crate::figma::models::export::ImageResponse;

pub fn get_image_urls(
    client: &Client,
    file_key: &str,
    node_ids: &[&str],
    format: &str,
    scale: f64,
) -> Result<ImageResponse, FigmaError> {
    let escaped: Vec<String> = node_ids.iter().map(|id| url_encode(id)).collect();
    let url = format!(
        "{}/v1/images/{}?ids={}&format={}&scale={}",
        client.base_url,
        file_key,
        escaped.join(","),
        format,
        scale
    );
    client.get(&url)
}

pub fn download_image(image_url: &str, output_path: &str) -> Result<(), FigmaError> {
    client::download_image(image_url, output_path)
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
