use regex::Regex;
use std::collections::HashMap;

/// Convert Confluence HTML to clean Markdown.
pub fn convert_to_markdown(raw_html: &str, base_url: &str) -> String {
    let base_url = base_url.trim_end_matches('/');

    let mut html = preprocess_html(raw_html, base_url);

    // Extract top-level tables (handling nesting) before html2md
    let mut tables: HashMap<String, String> = HashMap::new();
    let mut counter = 0;
    loop {
        match find_top_level_table(&html) {
            Some((start, end)) => {
                let key = format!("TBLPLACEHOLDER{}", counter);
                let table_html = &html[start..end].to_string();
                let table_md = table_to_markdown(table_html);
                tables.insert(key.clone(), table_md);
                html = format!("{}\n\n{}\n\n{}", &html[..start], key, &html[end..]);
                counter += 1;
            }
            None => break,
        }
    }

    // Convert remaining HTML to Markdown
    let mut md = html2md::parse_html(&html);

    // Restore table placeholders
    for (key, table_md) in &tables {
        md = md.replace(key, &format!("\n{}\n", table_md));
    }

    postprocess_markdown(&md)
}

/// Find the first top-level <table>...</table> handling nested tables.
fn find_top_level_table(html: &str) -> Option<(usize, usize)> {
    let open_re = Regex::new(r"<table[\s>]").unwrap();
    let close_re = Regex::new(r"</table>").unwrap();

    let start_match = open_re.find(html)?;
    let start = start_match.start();

    // Collect all open/close positions after start
    let mut events: Vec<(usize, bool)> = Vec::new(); // (pos, is_open)
    for m in open_re.find_iter(&html[start..]) {
        events.push((start + m.start(), true));
    }
    for m in close_re.find_iter(&html[start..]) {
        events.push((start + m.start(), false));
    }
    events.sort_by_key(|e| e.0);

    let mut depth = 0;
    for (pos, is_open) in &events {
        if *is_open {
            depth += 1;
        } else {
            depth -= 1;
            if depth == 0 {
                return Some((start, pos + 8)); // 8 = "</table>".len()
            }
        }
    }

    None
}

// ─── Table → Section Conversion ────────────────────────────────────────────

/// Check if a table is a Confluence page metadata header (Document Status, Owner, Version, etc.)
fn is_metadata_table(rows: &[Vec<TableCell>]) -> bool {
    let meta_keys = ["document status", "document owner", "platform", "version", "author", "created by"];
    let mut matches = 0;
    for row in rows {
        if row.len() >= 2 && row[0].is_header {
            let key = row[0].text.trim().to_lowercase();
            if meta_keys.iter().any(|k| key.contains(k)) {
                matches += 1;
            }
        }
    }
    matches >= 2
}

/// Extract useful fields from a metadata table (Figma link, Designer, MRD, etc.)
fn extract_metadata_fields(rows: &[Vec<TableCell>]) -> String {
    let keep_keys = ["figma", "designer", "mrd", "business owner", "request type", "change log"];
    let mut output = Vec::new();

    for row in rows {
        if row.len() >= 2 {
            let key = row[0].text.trim().to_lowercase();
            if keep_keys.iter().any(|k| key.contains(k)) {
                let label = row[0].text.trim();
                let value = row[1].text.trim();
                if !value.is_empty() {
                    output.push(format!("**{}:** {}", label, value));
                }
            }
        }
    }

    output.join("\n\n")
}

/// Convert an HTML table to markdown.
/// Multi-column data tables become pipe tables; 2-column key-value tables become bold key: value.
fn table_to_markdown(table_html: &str) -> String {
    let rows = extract_table_rows(table_html);
    if rows.is_empty() {
        return strip_tags(table_html);
    }

    // For Confluence page metadata tables, extract key fields instead of discarding entirely
    if is_metadata_table(&rows) {
        return extract_metadata_fields(&rows);
    }

    // Check if this is a multi-column data table (3+ columns, or has a header row)
    let has_header_row = !rows.is_empty() && rows[0].iter().all(|c| c.is_header);
    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);

    if max_cols >= 3 || (has_header_row && max_cols >= 2) {
        return table_to_pipe_table(&rows, max_cols, has_header_row);
    }

    // Fallback: 2-column key-value format
    let mut output = Vec::new();

    for row in &rows {
        if row.is_empty() {
            continue;
        }

        // Detect pattern: if first cell is a header (th) and second is value → key:value
        if row.len() == 2 && row[0].is_header && !row[0].text.is_empty() {
            let key = &row[0].text;
            let val = &row[1].text;
            if !val.is_empty() {
                output.push(format!("**{}:** {}", key, val));
            } else {
                output.push(format!("**{}**", key));
            }
            continue;
        }

        // For all other rows, emit each non-empty cell as a block
        for cell in row.iter() {
            let text = cell.text.trim();
            if text.is_empty() {
                continue;
            }
            output.push(text.to_string());
        }
    }

    output.join("\n\n")
}

/// Render rows as a markdown pipe table.
fn table_to_pipe_table(rows: &[Vec<TableCell>], max_cols: usize, has_header_row: bool) -> String {
    let mut lines = Vec::new();

    for (i, row) in rows.iter().enumerate() {
        let mut cells: Vec<String> = row
            .iter()
            .map(|c| {
                // Escape pipes in cell content and collapse newlines to spaces
                c.text
                    .trim()
                    .replace('\n', " ")
                    .replace('|', "\\|")
            })
            .collect();

        // Pad to max_cols if row has fewer cells
        while cells.len() < max_cols {
            cells.push(String::new());
        }

        lines.push(format!("| {} |", cells.join(" | ")));

        // Add separator after header row
        if i == 0 && has_header_row {
            let sep: Vec<&str> = (0..max_cols).map(|_| "---").collect();
            lines.push(format!("| {} |", sep.join(" | ")));
        }
    }

    // If no header row was detected, insert a separator after the first row anyway
    if !has_header_row && !lines.is_empty() {
        let sep: Vec<&str> = (0..max_cols).map(|_| "---").collect();
        lines.insert(1, format!("| {} |", sep.join(" | ")));
    }

    lines.join("\n")
}

struct TableCell {
    text: String,
    is_header: bool,
    #[allow(dead_code)]
    colspan: usize,
}

/// Extract rows from the outermost table only (depth 1 rows).
fn extract_table_rows(html: &str) -> Vec<Vec<TableCell>> {
    let mut rows = Vec::new();

    let re_tag = Regex::new(r"<(/?)(?:table|tr)\b[^>]*>").unwrap();
    let mut table_depth = 0;
    let mut tr_start: Option<usize> = None;

    for mat in re_tag.find_iter(html) {
        let tag = mat.as_str();
        if tag.starts_with("<table") {
            table_depth += 1;
        } else if tag.starts_with("</table") {
            table_depth -= 1;
        } else if tag.starts_with("<tr") && table_depth == 1 {
            tr_start = Some(mat.start());
        } else if tag.starts_with("</tr") && table_depth == 1 {
            if let Some(start) = tr_start {
                let tr_content = &html[start..mat.end()];
                let row = extract_cells(tr_content);
                if !row.is_empty() {
                    rows.push(row);
                }
                tr_start = None;
            }
        }
    }

    rows
}

fn extract_cells(tr_html: &str) -> Vec<TableCell> {
    let re_cell = Regex::new(r#"(?s)<(t[dh])\b([^>]*)>(.*?)</t[dh]>"#).unwrap();
    let re_colspan = Regex::new(r#"colspan="(\d+)""#).unwrap();
    let mut cells = Vec::new();

    for cap in re_cell.captures_iter(tr_html) {
        let tag_name = &cap[1]; // "th" or "td"
        let attrs = &cap[2];
        let cell_html = &cap[3];
        let colspan = re_colspan
            .captures(attrs)
            .and_then(|c| c[1].parse::<usize>().ok())
            .unwrap_or(1);
        let text = cell_to_text(cell_html);
        cells.push(TableCell {
            text,
            is_header: tag_name == "th",
            colspan,
        });
    }

    cells
}

/// Convert cell HTML content to clean Markdown text.
fn cell_to_text(html: &str) -> String {
    let mut text = html.to_string();

    // Convert nested tables to compact inline format
    if text.contains("<table") {
        loop {
            match find_top_level_table(&text) {
                Some((start, end)) => {
                    let nested = &text[start..end].to_string();
                    let nested_rows = extract_table_rows(nested);
                    // Render as compact rows: "Col1: Val1 | Col2: Val2"
                    let mut parts = Vec::new();
                    // Use first row as headers
                    let headers: Vec<String> = nested_rows
                        .first()
                        .map(|r| r.iter().map(|c| c.text.clone()).collect())
                        .unwrap_or_default();
                    for row in nested_rows.iter().skip(1) {
                        let vals: Vec<String> = row.iter().enumerate().map(|(i, c)| {
                            let header = headers.get(i).map(|h| h.as_str()).unwrap_or("");
                            let val = c.text.trim();
                            if val.is_empty() {
                                return String::new();
                            }
                            if header.is_empty() {
                                val.to_string()
                            } else {
                                format!("{}: {}", header, val)
                            }
                        }).filter(|s| !s.is_empty()).collect();
                        if !vals.is_empty() {
                            parts.push(vals.join(" | "));
                        }
                    }
                    let nested_md = parts.join("\n");
                    text = format!("{}\n{}\n{}", &text[..start], nested_md, &text[end..]);
                }
                None => break,
            }
        }
    }

    // Convert images to markdown (each on its own line)
    let re_img = Regex::new(r#"<img[^>]*\bsrc="([^"]+)"[^>]*>"#).unwrap();
    text = re_img
        .replace_all(&text, |caps: &regex::Captures| {
            let src = &caps[1];
            let alt = extract_attr(&caps[0], "alt").unwrap_or_default();
            format!("\n\n![{}]({})\n\n", alt, src)
        })
        .to_string();

    // Convert links
    let re_link = Regex::new(r#"(?s)<a[^>]*\bhref="([^"]+)"[^>]*>(.*?)</a>"#).unwrap();
    text = re_link
        .replace_all(&text, |caps: &regex::Captures| {
            let href = &caps[1];
            let link_text = strip_tags(&caps[2]).trim().to_string();
            if link_text.is_empty() {
                href.to_string()
            } else {
                format!("[{}]({})", link_text, href)
            }
        })
        .to_string();

    // Convert headings to ## headings
    let re_h = Regex::new(r"(?s)<(h)(\d)[^>]*>(.*?)</h\d>").unwrap();
    text = re_h
        .replace_all(&text, |caps: &regex::Captures| {
            let level: usize = caps[2].parse().unwrap_or(3);
            let content = strip_tags(&caps[3]).trim().to_string();
            format!("\n\n{} {}\n\n", "#".repeat(level), content)
        })
        .to_string();

    // Convert <strong>/<b>
    text = Regex::new(r"(?s)<(?:strong|b)>(.*?)</(?:strong|b)>")
        .unwrap()
        .replace_all(&text, "**$1**")
        .to_string();

    // Convert <em>/<i>
    text = Regex::new(r"(?s)<(?:em|i)>(.*?)</(?:em|i)>")
        .unwrap()
        .replace_all(&text, "*$1*")
        .to_string();

    // Convert <br> to newlines
    text = Regex::new(r"<br\s*/?>")
        .unwrap()
        .replace_all(&text, "\n")
        .to_string();

    // Convert list items
    text = Regex::new(r"(?s)<li[^>]*>(.*?)</li>")
        .unwrap()
        .replace_all(&text, |caps: &regex::Captures| {
            let content = strip_tags(&caps[1]).trim().to_string();
            format!("\n- {}", content)
        })
        .to_string();

    // Convert <p> to paragraph breaks
    text = Regex::new(r"(?s)<p[^>]*>(.*?)</p>")
        .unwrap()
        .replace_all(&text, "\n$1\n")
        .to_string();

    // Strip remaining tags
    text = strip_tags(&text);

    // Decode entities
    text = decode_entities(&text);
    text = text.replace("\u{00a0}", " ");

    // Clean up whitespace: collapse multiple blank lines, trim lines
    let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
    text = lines.join("\n");
    let re_blank = Regex::new(r"\n{3,}").unwrap();
    text = re_blank.replace_all(&text, "\n\n").to_string();

    text.trim().to_string()
}

fn strip_tags(html: &str) -> String {
    Regex::new(r"<[^>]+>")
        .unwrap()
        .replace_all(html, "")
        .to_string()
}

fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    let pattern = format!(r#"{}="([^"]*)""#, attr_name);
    let re = Regex::new(&pattern).ok()?;
    re.captures(tag).map(|c| c[1].to_string())
}

// ─── HTML Preprocessing ────────────────────────────────────────────────────

fn preprocess_html(html: &str, base_url: &str) -> String {
    let mut html = html.to_string();

    // Resolve relative URLs to absolute
    if !base_url.is_empty() {
        html = html.replace(r#"src="/download/"#, &format!(r#"src="{}/download/"#, base_url));
        html = html.replace(r#"src="/rest/"#, &format!(r#"src="{}/rest/"#, base_url));
        html = html.replace(
            r#"data-image-src="/download/"#,
            &format!(r#"data-image-src="{}/download/"#, base_url),
        );
        html = html.replace(r#"href="/display/"#, &format!(r#"href="{}/display/"#, base_url));
        html = html.replace(r#"href="/download/"#, &format!(r#"href="{}/download/"#, base_url));
        html = html.replace(r#"href="/pages/"#, &format!(r#"href="{}/pages/"#, base_url));
    }

    // Convert Confluence syntax-highlighted code blocks
    let re_pre = Regex::new(r#"(?s)<pre class="syntaxhighlighter-pre"[^>]*>(.*?)</pre>"#).unwrap();
    html = re_pre
        .replace_all(&html, |caps: &regex::Captures| {
            let mut content = caps[1].to_string();
            content = decode_entities(&content);
            let mut lang = String::new();
            let lines: Vec<&str> = content.splitn(2, '\n').collect();
            if is_known_lang(lines[0].trim()) {
                lang = lines[0].trim().to_string();
                content = lines.get(1).unwrap_or(&"").to_string();
            }
            format!("\n\n```{}\n{}\n```\n\n", lang, content.trim())
        })
        .to_string();

    // Convert status macros
    let re_status = Regex::new(r#"<span[^>]*class="[^"]*status-macro[^"]*"[^>]*>([^<]*)</span>"#).unwrap();
    html = re_status.replace_all(&html, "**[$1]**").to_string();

    // Extract user mentions
    let re_user = Regex::new(r#"<a[^>]*class="[^"]*user-mention[^"]*"[^>]*>([^<]+)</a>"#).unwrap();
    html = re_user.replace_all(&html, "**$1**").to_string();

    // Remove TOC macros
    let re_toc = Regex::new(r#"(?s)<div[^>]*class="[^"]*toc-macro[^"]*"[^>]*>.*?</div>"#).unwrap();
    html = re_toc.replace_all(&html, "").to_string();

    // Remove color spans
    let re_color = Regex::new(r#"<span\s+style="color:\s*rgb\([^)]+\);?[^"]*">"#).unwrap();
    html = re_color.replace_all(&html, "").to_string();

    // Remove mtk spans
    let re_mtk = Regex::new(r#"<span[^>]*class="mtk\d+"[^>]*>"#).unwrap();
    html = re_mtk.replace_all(&html, "").to_string();

    // Clean wrapper divs
    html = html.replace(r#"<div class="content-wrapper">"#, "");
    html = html.replace(r#"<div class="table-wrap">"#, "");
    let re_panel = Regex::new(r#"<div[^>]*class="code panel[^"]*"[^>]*>"#).unwrap();
    html = re_panel.replace_all(&html, "").to_string();
    let re_code_content = Regex::new(r#"<div[^>]*class="codeContent[^"]*"[^>]*>"#).unwrap();
    html = re_code_content.replace_all(&html, "").to_string();

    // Simplify Confluence images
    let re_img = Regex::new(r#"(?s)<img[^>]*\bsrc="([^"]+)"[^>]*>"#).unwrap();
    html = re_img
        .replace_all(&html, |caps: &regex::Captures| {
            let src = &caps[1];
            let alt = extract_attr(&caps[0], "alt")
                .or_else(|| extract_attr(&caps[0], "data-linked-resource-default-alias"))
                .unwrap_or_default();
            format!(r#"<img src="{}" alt="{}">"#, src, alt)
        })
        .to_string();

    // Convert info/note macros to blockquotes
    let re_macro = Regex::new(r#"<div[^>]*class="[^"]*confluence-information-macro[^"]*"[^>]*>"#).unwrap();
    html = re_macro.replace_all(&html, "<blockquote>").to_string();

    html
}

fn decode_entities(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
}

// ─── Markdown Postprocessing ───────────────────────────────────────────────

fn postprocess_markdown(md: &str) -> String {
    let mut md = md.to_string();

    // Clean entities
    md = decode_entities(&md);
    md = md.replace("\u{00a0}", " ");

    // Remove bold inside headings
    let re_bold_heading = Regex::new(r"(?m)^(#{1,6}\s+)\*\*(.+?)\*\*\s*$").unwrap();
    md = re_bold_heading.replace_all(&md, "${1}${2}").to_string();

    // Fix escaped markdown outside code blocks
    let lines: Vec<&str> = md.split('\n').collect();
    let mut in_code = false;
    let mut processed: Vec<String> = Vec::with_capacity(lines.len());
    for line in &lines {
        if line.starts_with("```") {
            in_code = !in_code;
            processed.push(line.to_string());
            continue;
        }
        if in_code {
            processed.push(line.to_string());
            continue;
        }
        let mut l = line.replace("\\[", "[");
        l = l.replace("\\]", "]");
        l = l.replace("\\*", "*");
        l = l.replace("\\\\", "\\");
        l = l.trim_end().to_string();
        processed.push(l);
    }
    md = processed.join("\n");

    // Clean leftover tags
    let re_leftover = Regex::new(r"</(?:span|div|colgroup|col|thead|tbody)>").unwrap();
    md = re_leftover.replace_all(&md, "").to_string();
    let re_leftover2 = Regex::new(r#"<(?:colgroup|col|thead|tbody)[^>]*>"#).unwrap();
    md = re_leftover2.replace_all(&md, "").to_string();

    // Remove trailing quotes from image artifacts
    md = md.replace(")\u{0022}", ")");

    // Collapse blank lines
    let re_blank = Regex::new(r"\n{3,}").unwrap();
    md = re_blank.replace_all(&md, "\n\n").to_string();

    md.trim().to_string()
}

/// Lightweight HTML → markdown. Reuses Confluence preprocessing and table extraction
/// for proper pipe tables, but skips html2md and section remapping to preserve original structure.
pub fn convert_to_lightweight_markdown(raw_html: &str, base_url: &str) -> String {
    let base_url = base_url.trim_end_matches('/');
    let mut html = preprocess_html(raw_html, base_url);

    // Extract and convert tables to pipe tables (reusing existing robust logic)
    let mut tables: Vec<(String, String)> = Vec::new();
    let mut counter = 0;
    loop {
        match find_top_level_table(&html) {
            Some((start, end)) => {
                let key = format!("__TBLRAW{}__", counter);
                let table_html = &html[start..end].to_string();
                let table_md = table_to_markdown(table_html);
                tables.push((key.clone(), table_md));
                html = format!("{}\n\n{}\n\n{}", &html[..start], key, &html[end..]);
                counter += 1;
            }
            None => break,
        }
    }

    // Headings
    let re_h = Regex::new(r"(?is)<h([1-6])[^>]*>(.*?)</h[1-6]>").unwrap();
    html = re_h.replace_all(&html, |caps: &regex::Captures| {
        let level: usize = caps[1].parse().unwrap_or(2);
        let hashes = "#".repeat(level);
        let text = strip_tags(&caps[2]);
        format!("\n{} {}\n", hashes, text.trim())
    }).to_string();

    // Bold
    let re_bold = Regex::new(r"(?is)<(strong|b)\b[^>]*>(.*?)</(strong|b)>").unwrap();
    html = re_bold.replace_all(&html, "**$2**").to_string();

    // Italic
    let re_italic = Regex::new(r"(?is)<(em|i)\b[^>]*>(.*?)</(em|i)>").unwrap();
    html = re_italic.replace_all(&html, "*$2*").to_string();

    // Strikethrough
    let re_strike = Regex::new(r"(?is)<(del|s|strike)\b[^>]*>(.*?)</(del|s|strike)>").unwrap();
    html = re_strike.replace_all(&html, "~~$2~~").to_string();

    // Inline code
    let re_code = Regex::new(r"(?is)<code>(.*?)</code>").unwrap();
    html = re_code.replace_all(&html, "`$1`").to_string();

    // Code blocks
    let re_pre = Regex::new(r"(?is)<pre[^>]*>(.*?)</pre>").unwrap();
    html = re_pre.replace_all(&html, "\n```\n$1\n```\n").to_string();

    // <br> → newline
    let re_br = Regex::new(r"(?i)<br\s*/?>").unwrap();
    html = re_br.replace_all(&html, "\n").to_string();

    // List items
    let re_li = Regex::new(r"(?is)<li[^>]*>(.*?)</li>").unwrap();
    html = re_li.replace_all(&html, "\n- $1").to_string();

    // Paragraphs/divs → newlines
    let re_block_open = Regex::new(r"(?i)<(p|div)\b[^>]*>").unwrap();
    html = re_block_open.replace_all(&html, "\n").to_string();
    let re_block_close = Regex::new(r"(?i)</(p|div)>").unwrap();
    html = re_block_close.replace_all(&html, "\n").to_string();

    // Horizontal rule
    let re_hr = Regex::new(r"(?i)<hr\s*/?>").unwrap();
    html = re_hr.replace_all(&html, "\n---\n").to_string();

    // Images
    let re_img = Regex::new(r#"<img[^>]*?src="([^"]*)"[^>]*?alt="([^"]*)"[^>]*/?\s*>"#).unwrap();
    html = re_img.replace_all(&html, "![$2]($1)").to_string();
    let re_img2 = Regex::new(r#"<img[^>]*?alt="([^"]*)"[^>]*?src="([^"]*)"[^>]*/?\s*>"#).unwrap();
    html = re_img2.replace_all(&html, "![$1]($2)").to_string();
    let re_img3 = Regex::new(r#"<img[^>]*?src="([^"]*)"[^>]*/?\s*>"#).unwrap();
    html = re_img3.replace_all(&html, "![]($1)").to_string();

    // Links
    let re_link = Regex::new(r#"(?s)<a[^>]*?href="([^"]*)"[^>]*>(.*?)</a>"#).unwrap();
    html = re_link.replace_all(&html, "[$2]($1)").to_string();

    // Blockquotes
    let re_bq = Regex::new(r"(?i)<blockquote[^>]*>").unwrap();
    html = re_bq.replace_all(&html, "\n> ").to_string();
    let re_bq_close = Regex::new(r"(?i)</blockquote>").unwrap();
    html = re_bq_close.replace_all(&html, "\n").to_string();

    // Strip all remaining tags
    html = strip_tags(&html);

    // Restore table placeholders
    for (key, table_md) in &tables {
        html = html.replace(key, &format!("\n{}\n", table_md));
    }

    // Decode entities
    html = decode_entities(&html);
    html = html.replace("\u{00a0}", " ");

    // Collapse blank lines
    let re_blank = Regex::new(r"\n{3,}").unwrap();
    html = re_blank.replace_all(&html, "\n\n").to_string();

    // Trim each line
    html = html.lines().map(|l| l.trim()).collect::<Vec<_>>().join("\n");

    // Final blank line collapse
    let re_blank2 = Regex::new(r"\n{3,}").unwrap();
    html = re_blank2.replace_all(&html, "\n\n").to_string();

    html.trim().to_string()
}

fn is_known_lang(s: &str) -> bool {
    matches!(
        s,
        "json" | "bash" | "yaml" | "xml" | "java" | "go" | "python"
        | "javascript" | "typescript" | "sql" | "shell" | "sh" | "html"
        | "css" | "groovy" | "kotlin" | "swift" | "ruby" | "php"
        | "text" | "plain" | "properties" | "toml"
    )
}
