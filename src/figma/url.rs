use crate::figma::error::FigmaError;
use serde::Serialize;

const REST_API_V1: &str = "https://api.figma.com/v1";

#[derive(Debug, Serialize)]
pub struct ParsedFigmaUrl {
    pub input: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_type: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub is_branch: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_file_key: Option<String>,
    pub file_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_slug: Option<String>,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub query: std::collections::HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sharing_view_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<NodeFromUrl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment: Option<String>,
    pub links: ParsedLinks,
}

fn is_false(b: &bool) -> bool {
    !b
}

#[derive(Debug, Serialize)]
pub struct NodeFromUrl {
    pub raw: String,
    pub api_format: String,
    pub url_encoded: String,
}

#[derive(Debug, Serialize)]
pub struct ParsedLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<String>,
    pub figma_design: String,
    pub rest_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_nodes: Option<String>,
}

pub struct FigmaUrl {
    pub file_key: String,
    pub node_id: String,
    #[allow(dead_code)]
    pub url_type: String,
}

pub fn parse(input: &str) -> Result<FigmaUrl, FigmaError> {
    let d = parse_detailed(input)?;
    let node_id = d.node.as_ref().map(|n| n.api_format.clone()).unwrap_or_default();
    let url_type = if d.kind == "file_key" {
        String::new()
    } else {
        d.url_type.unwrap_or_default()
    };
    Ok(FigmaUrl {
        file_key: d.file_key,
        node_id,
        url_type,
    })
}

pub fn file_key(input: &str) -> String {
    match parse(input) {
        Ok(parsed) => parsed.file_key,
        Err(_) => input.to_string(),
    }
}

pub fn parse_detailed(input: &str) -> Result<ParsedFigmaUrl, FigmaError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(FigmaError::Parse("empty input".to_string()));
    }

    if !input.contains("figma.com") {
        return Ok(plain_file_key_parsed(input));
    }

    // Manual URL parsing
    let (scheme, rest) = if let Some(idx) = input.find("://") {
        (&input[..idx], &input[idx + 3..])
    } else {
        return Err(FigmaError::Parse("invalid URL: missing scheme".to_string()));
    };

    let (host_and_path, fragment) = match rest.find('#') {
        Some(idx) => (&rest[..idx], Some(rest[idx + 1..].to_string())),
        None => (rest, None),
    };

    let (host_and_path_no_query, query_string) = match host_and_path.find('?') {
        Some(idx) => (&host_and_path[..idx], Some(&host_and_path[idx + 1..])),
        None => (host_and_path, None),
    };

    let (host, path) = match host_and_path_no_query.find('/') {
        Some(idx) => (
            &host_and_path_no_query[..idx],
            &host_and_path_no_query[idx..],
        ),
        None => {
            return Err(FigmaError::Parse("invalid URL: missing path".to_string()));
        }
    };

    let trimmed_path = path.trim_matches('/');
    let parts: Vec<&str> = trimmed_path.split('/').collect();
    if parts.len() < 2 {
        return Err(FigmaError::Parse(
            "cannot parse Figma URL: expected /design/:fileKey/... or /file/:fileKey/...".to_string(),
        ));
    }

    let url_type = parts[0];
    if !["design", "file", "board", "proto"].contains(&url_type) {
        return Err(FigmaError::Parse(format!(
            "cannot parse Figma URL: unsupported path type {:?}",
            url_type
        )));
    }

    let mut query = std::collections::HashMap::new();
    if let Some(qs) = query_string {
        for pair in qs.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                let v_decoded = url_decode(v);
                if !v_decoded.is_empty() {
                    query.insert(k.to_string(), v_decoded);
                }
            }
        }
    }

    let (is_branch, parent_file_key, file_key_val, file_slug) =
        if parts.len() >= 4 && parts[2] == "branch" {
            let slug = if parts.len() > 4 {
                Some(path_join_unescape(&parts[4..]))
            } else {
                None
            };
            (true, Some(parts[1].to_string()), parts[3].to_string(), slug)
        } else {
            let slug = if parts.len() > 2 {
                Some(path_join_unescape(&parts[2..]))
            } else {
                None
            };
            (false, None, parts[1].to_string(), slug)
        };

    if file_key_val.is_empty() {
        return Err(FigmaError::Parse(
            "cannot extract file key from URL".to_string(),
        ));
    }

    let node = query.get("node-id").map(|node_param| {
        let api_fmt = node_id_url_to_api(node_param);
        let url_encoded = url_encode(&api_fmt);
        NodeFromUrl {
            raw: node_param.clone(),
            api_format: api_fmt,
            url_encoded,
        }
    });

    let sharing_view_token = query.get("t").cloned();

    // Normalize web URL
    let normalized_host = match host {
        "figma.com" => "www.figma.com",
        _ => host,
    };
    let web = {
        let mut web_url = format!("https://{}{}", normalized_host, path);
        if let Some(qs) = query_string {
            web_url.push('?');
            web_url.push_str(qs);
        }
        if let Some(ref frag) = fragment {
            web_url.push('#');
            web_url.push_str(frag);
        }
        web_url
    };

    let links = build_links(url_type, &file_key_val, file_slug.as_deref(), node.as_ref(), &web);

    Ok(ParsedFigmaUrl {
        input: input.to_string(),
        kind: "url".to_string(),
        scheme: Some(scheme.to_string()),
        host: Some(host.to_string()),
        path: Some(path.to_string()),
        url_type: Some(url_type.to_string()),
        is_branch,
        parent_file_key,
        file_key: file_key_val,
        file_slug,
        query,
        sharing_view_token,
        node,
        fragment,
        links,
    })
}

fn plain_file_key_parsed(key: &str) -> ParsedFigmaUrl {
    ParsedFigmaUrl {
        input: key.to_string(),
        kind: "file_key".to_string(),
        scheme: None,
        host: None,
        path: None,
        url_type: None,
        is_branch: false,
        parent_file_key: None,
        file_key: key.to_string(),
        file_slug: None,
        query: std::collections::HashMap::new(),
        sharing_view_token: None,
        node: None,
        fragment: None,
        links: build_links("design", key, None, None, ""),
    }
}

fn build_links(
    url_type: &str,
    file_key: &str,
    slug: Option<&str>,
    node: Option<&NodeFromUrl>,
    web: &str,
) -> ParsedLinks {
    let prefix = figma_web_path_prefix(url_type);
    let design = match slug {
        Some(s) => format!("https://www.figma.com/{}/{}/{}", prefix, file_key, s),
        None => format!("https://www.figma.com/{}/{}", prefix, file_key),
    };
    let rest_nodes = node.map(|n| {
        format!(
            "{}/files/{}/nodes?ids={}",
            REST_API_V1, file_key, n.url_encoded
        )
    });
    ParsedLinks {
        web: if web.is_empty() {
            None
        } else {
            Some(web.to_string())
        },
        figma_design: design,
        rest_file: format!("{}/files/{}", REST_API_V1, file_key),
        rest_nodes,
    }
}

fn figma_web_path_prefix(url_type: &str) -> &str {
    match url_type {
        "file" | "board" | "proto" => url_type,
        _ => "design",
    }
}

fn path_join_unescape(segments: &[&str]) -> String {
    segments
        .iter()
        .map(|s| url_decode(s))
        .collect::<Vec<_>>()
        .join("/")
}

fn node_id_url_to_api(s: &str) -> String {
    s.replace('-', ":")
}

fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte_val) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                result.push(byte_val as char);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
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
