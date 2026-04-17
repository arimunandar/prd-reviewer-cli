use crate::jira::client::Client;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Download all wiki/jira images in markdown, cache them locally, replace URLs.
pub fn download_images(md: &str, page_id: &str, client: &Client) -> String {
    let cache_dir = image_cache_dir(page_id);
    let re = regex::Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();

    re.replace_all(md, |caps: &regex::Captures| {
        let alt = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let url = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        if !is_downloadable_image_url(url) {
            return format!("![{}]({})", alt, url);
        }

        match download_image(url, &cache_dir, client) {
            Ok(local_path) => format!("![{}]({})", alt, local_path),
            Err(_) => format!("![{}]({})", alt, url),
        }
    })
    .to_string()
}

fn is_downloadable_image_url(url: &str) -> bool {
    url.contains("/download/attachments/")
        || url.contains("/download/thumbnails/")
        || url.contains("/rest/api/content/")
        || url.contains("/secure/attachment/")
        || url.contains("/rest/api/2/attachment/")
}

/// Cache images under `<project>/.prd-reviewer/images/<page_id>/` when a project
/// root is detectable, otherwise fall back to `$TMPDIR/prd-reviewer/images/<page_id>/`.
fn image_cache_dir(page_id: &str) -> PathBuf {
    if let Some(root) = find_project_root() {
        return root.join(".prd-reviewer").join("images").join(page_id);
    }
    std::env::temp_dir()
        .join("prd-reviewer")
        .join("images")
        .join(page_id)
}

fn find_project_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join(".prd-reviewer").exists() || dir.join(".claude").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn download_image(url: &str, cache_dir: &PathBuf, client: &Client) -> Result<String, String> {
    let filename = image_filename(url);
    fs::create_dir_all(cache_dir).map_err(|e| e.to_string())?;

    let local_path = cache_dir.join(&filename);
    let local_str = local_path.to_string_lossy().to_string();

    // Skip if cached
    if local_path.exists() {
        return Ok(local_str);
    }

    // Download with auth
    let agent = client.raw_agent();
    let mut req = agent.get(url);
    if let Some((key, val)) = client.auth_for_url(url) {
        req = req.set(&key, &val);
    }

    let resp = req.call().map_err(|e| e.to_string())?;
    let mut reader = resp.into_reader();
    let mut file = fs::File::create(&local_path).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;

    Ok(local_str)
}

fn image_filename(url: &str) -> String {
    let parts: Vec<&str> = url.split('/').collect();

    // Wiki format: .../download/attachments/{pageId}/{filename}?...
    for (i, p) in parts.iter().enumerate() {
        if (*p == "attachments" || *p == "thumbnails") && i + 2 < parts.len() {
            let name = parts[i + 2].split('?').next().unwrap_or(parts[i + 2]);
            return name.to_string();
        }
    }

    // Last segment with extension
    if let Some(last) = parts.last() {
        let name = last.split('?').next().unwrap_or(last);
        if !name.is_empty() && name.contains('.') {
            return name.to_string();
        }
    }

    // Fallback: hash URL
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hasher.finalize();
    let hex: String = hash[..8].iter().map(|b| format!("{:02x}", b)).collect();
    format!("{}.png", hex)
}
