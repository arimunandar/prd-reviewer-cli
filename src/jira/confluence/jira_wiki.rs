use std::collections::HashMap;

/// Map of filename → download URL for Jira issue attachments.
pub type AttachmentMap = HashMap<String, String>;

/// Convert Jira wiki markup to Markdown.
pub fn convert_jira_wiki(wiki: &str, attachments: &AttachmentMap) -> String {
    let lines: Vec<String> = wiki.lines().map(|l| convert_jira_line(l)).collect();
    let mut md = lines.join("\n");
    md = convert_inline_formatting(&md, attachments);

    // Clean up excessive blank lines
    while md.contains("\n\n\n\n") {
        md = md.replace("\n\n\n\n", "\n\n\n");
    }

    md.trim().to_string()
}

fn convert_jira_line(line: &str) -> String {
    let trimmed = line.trim();

    // Headings: h1. through h6.
    for i in (1..=6).rev() {
        let prefix = format!("h{}. ", i);
        if trimmed.starts_with(&prefix) {
            return format!("{} {}", "#".repeat(i), &trimmed[prefix.len()..]);
        }
    }

    // Unordered list items: * item, ** item
    if let Some(rest) = strip_list_prefix(trimmed, '*') {
        let (depth, text) = rest;
        let indent = "  ".repeat(depth);
        return format!("{}- {}", indent, text);
    }

    // Ordered list items: # item, ## item
    if let Some(rest) = strip_list_prefix(trimmed, '#') {
        let (depth, text) = rest;
        let indent = "  ".repeat(depth);
        return format!("{}1. {}", indent, text);
    }

    // Horizontal rule
    if trimmed == "----" || trimmed == "---" {
        return "\n---\n".to_string();
    }

    // Blockquote: bq. text
    if let Some(text) = trimmed.strip_prefix("bq. ") {
        return format!("> {}", text);
    }

    line.to_string()
}

fn strip_list_prefix(s: &str, marker: char) -> Option<(usize, &str)> {
    let bytes = s.as_bytes();
    let mut count = 0;
    for &b in bytes {
        if b == marker as u8 {
            count += 1;
        } else {
            break;
        }
    }
    if count > 0 && count < s.len() && bytes[count] == b' ' {
        let depth = count - 1;
        Some((depth, &s[count + 1..]))
    } else {
        None
    }
}

fn convert_inline_formatting(md: &str, attachments: &AttachmentMap) -> String {
    let mut result = md.to_string();

    // Images: !filename.png! or !filename.png|params!
    result = regex_replace_all(
        &result,
        r"!([^!\n|]+?)(?:\|[^!]*)?\!",
        |caps: &[&str]| {
            let filename = caps[1];
            if filename.starts_with("http") {
                return caps[0].to_string();
            }
            let url = attachments.get(filename).cloned().unwrap_or_else(|| filename.to_string());
            format!("![{}]({})", filename, url)
        },
    );

    // Links: [text|url]
    result = regex_replace_all(
        &result,
        r"\[([^|\]\n]+)\|([^\]\n]+)\]",
        |caps: &[&str]| format!("[{}]({})", caps[1], caps[2]),
    );

    // Links: [url]
    result = regex_replace_all(
        &result,
        r"\[(https?://[^\]\n]+)\]",
        |caps: &[&str]| format!("[{}]({})", caps[1], caps[1]),
    );

    // Monospace: {{text}}
    result = regex_replace_all(
        &result,
        r"\{\{([^}\n]+?)\}\}",
        |caps: &[&str]| format!("`{}`", caps[1]),
    );

    // Code block: {code:lang}...{code}
    result = regex_replace_all_dotall(
        &result,
        r"\{code(?::([^}]*))?\}(.*?)\{code\}",
        |caps: &[&str]| {
            let params = caps.get(1).unwrap_or(&"");
            let mut lang = "";
            if params.contains("language=") {
                if let Some(start) = params.find("language=") {
                    let rest = &params[start + 9..];
                    lang = rest.split(|c: char| !c.is_alphanumeric()).next().unwrap_or("");
                }
            }
            let code = caps.get(2).unwrap_or(&"");
            format!("\n```{}{}\n```\n", lang, code)
        },
    );

    // {noformat}...{noformat}
    result = regex_replace_all_dotall(
        &result,
        r"\{noformat\}(.*?)\{noformat\}",
        |caps: &[&str]| format!("\n```\n{}\n```\n", caps[1]),
    );

    // Quote blocks: {quote}...{quote}
    result = regex_replace_all_dotall(
        &result,
        r"\{quote\}(.*?)\{quote\}",
        |caps: &[&str]| {
            let content = caps[1].trim();
            let quoted: Vec<String> = content.lines().map(|l| format!("> {}", l)).collect();
            format!("\n{}\n", quoted.join("\n"))
        },
    );

    // Panels: {panel}...{panel}
    result = regex_replace_all_dotall(
        &result,
        r"\{panel(?::[^}]*)?\}(.*?)\{panel\}",
        |caps: &[&str]| format!("\n> {}\n", caps[1]),
    );

    // Color: {color:xxx}text{color}
    result = regex_replace_all(
        &result,
        r"\{color:[^}]+\}(.*?)\{color\}",
        |caps: &[&str]| caps[1].to_string(),
    );

    result
}

// Simple regex helpers using the regex crate (already a transitive dep)
fn regex_replace_all(input: &str, pattern: &str, replacer: impl Fn(&[&str]) -> String) -> String {
    let re = regex::Regex::new(pattern).unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let groups: Vec<&str> = (0..caps.len())
            .map(|i| caps.get(i).map(|m| m.as_str()).unwrap_or(""))
            .collect();
        replacer(&groups)
    })
    .to_string()
}

fn regex_replace_all_dotall(input: &str, pattern: &str, replacer: impl Fn(&[&str]) -> String) -> String {
    let re = regex::Regex::new(&format!("(?s){}", pattern)).unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let groups: Vec<&str> = (0..caps.len())
            .map(|i| caps.get(i).map(|m| m.as_str()).unwrap_or(""))
            .collect();
        replacer(&groups)
    })
    .to_string()
}
