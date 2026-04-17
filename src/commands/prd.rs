use clap::Subcommand;
use serde::Serialize;

use crate::jira::api::wiki_api;
use crate::jira::client::Client;
use crate::jira::config::Config;
use crate::jira::confluence;
use crate::jira::error::JiraError;

// ─── Unified PRD Review Framework ──────────────────────────────────────────
// Single source of truth for both CLI automated review and AI agent skill.
// 14 mandatory sections, weighted by automation impact.

#[allow(dead_code)]
struct SectionSpec {
    id: u8,
    name: &'static str,
    weight_missing: u32,
    weight_incomplete: u32,
    automation_impact: &'static str,
}

const PRD_SECTIONS: &[SectionSpec] = &[
    SectionSpec { id: 1,  name: "Metadata",                      weight_missing: 5,  weight_incomplete: 2, automation_impact: "Routing: team, urgency, status" },
    SectionSpec { id: 2,  name: "Background & Problem Statement", weight_missing: 10, weight_incomplete: 3, automation_impact: "Context for AI to understand why" },
    SectionSpec { id: 3,  name: "Objectives",                     weight_missing: 10, weight_incomplete: 3, automation_impact: "Measurable goals for acceptance" },
    SectionSpec { id: 4,  name: "Scope",                          weight_missing: 8,  weight_incomplete: 3, automation_impact: "Prevents AI scope creep" },
    SectionSpec { id: 5,  name: "General Rules",                  weight_missing: 5,  weight_incomplete: 2, automation_impact: "Global constraints AI must follow" },
    SectionSpec { id: 6,  name: "Acceptance Criteria",            weight_missing: 15, weight_incomplete: 5, automation_impact: "Direct test generation input" },
    SectionSpec { id: 7,  name: "Feature Sections",               weight_missing: 15, weight_incomplete: 5, automation_impact: "Core implementation instructions" },
    SectionSpec { id: 8,  name: "Design Reference",               weight_missing: 8,  weight_incomplete: 3, automation_impact: "UI implementation source of truth" },
    SectionSpec { id: 9,  name: "User Flows",                     weight_missing: 8,  weight_incomplete: 3, automation_impact: "Navigation and state machine input" },
    SectionSpec { id: 10, name: "Technical Docs / API Contract",  weight_missing: 8,  weight_incomplete: 3, automation_impact: "Backend integration automation" },
    SectionSpec { id: 11, name: "Event Tracking",                 weight_missing: 3,  weight_incomplete: 1, automation_impact: "Analytics automation" },
    SectionSpec { id: 12, name: "LCMP Keys",                      weight_missing: 3,  weight_incomplete: 1, automation_impact: "Localization automation" },
    SectionSpec { id: 13, name: "Success Metrics",                weight_missing: 5,  weight_incomplete: 2, automation_impact: "Post-launch validation" },
    SectionSpec { id: 14, name: "Related PRDs",                   weight_missing: 2,  weight_incomplete: 1, automation_impact: "Dependency awareness" },
];

const APPROVAL_THRESHOLD: u32 = 95;

// ─── Serializable Output Structs ──────────────────────────────────────────

#[derive(Serialize)]
struct ReviewOutput {
    page_id: String,
    title: String,
    score: u32,
    threshold: u32,
    decision: String,
    findings: Vec<FindingOutput>,
}

#[derive(Serialize)]
struct FindingOutput {
    severity: String,
    section: String,
    points: u32,
    message: String,
}

#[derive(Subcommand)]
pub enum PrdCommands {
    /// Fetch wiki page and format as structured PRD
    Fetch {
        /// Wiki page ID
        page_id: String,
        /// Also run quality review
        #[arg(long)]
        review: bool,
        /// Post review findings as comment on wiki page
        #[arg(long)]
        comment: bool,
        /// Output raw HTML without markdown conversion (preserves all original content)
        #[arg(long)]
        raw: bool,
        /// Output review as JSON (machine-readable)
        #[arg(long)]
        json: bool,
        /// Use relaxed scoring (default is strict 14-section weights)
        #[arg(long)]
        relaxed: bool,
        /// Skip TLS verification
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Review a fetched PRD for missing sections (14-section framework)
    Review {
        /// Wiki page ID
        page_id: String,
        /// Post review findings as comment on wiki page
        #[arg(long)]
        comment: bool,
        /// Output as JSON (machine-readable)
        #[arg(long)]
        json: bool,
        /// Use relaxed scoring (default is strict 14-section weights)
        #[arg(long)]
        relaxed: bool,
        /// Skip TLS verification
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Output the standard PRD template for Product Team (v2 — 14 sections)
    Template,
}

pub fn run(cmd: PrdCommands) {
    if let Err(e) = run_inner(cmd) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_inner(cmd: PrdCommands) -> Result<(), JiraError> {
    match cmd {
        PrdCommands::Fetch { page_id, review, comment, raw, json, relaxed, insecure } => {
            let strict = !relaxed;
            if raw {
                let prd = fetch_raw(&page_id, insecure)?;
                println!("{}", prd.content);
                save_prd_raw(&prd.title, &prd.content);
                return Ok(());
            }
            let prd = fetch_and_format(&page_id, insecure)?;
            println!("{}", prd.content);
            save_prd(&prd.title, &prd.content);
            if review {
                do_review(&page_id, &prd.title, &prd.content, &prd.raw_md, comment, json, strict, insecure)?;
            }
            Ok(())
        }
        PrdCommands::Review { page_id, comment, json, relaxed, insecure } => {
            let strict = !relaxed;
            let prd = fetch_and_format(&page_id, insecure)?;
            save_prd(&prd.title, &prd.content);
            do_review(&page_id, &prd.title, &prd.content, &prd.raw_md, comment, json, strict, insecure)
        }
        PrdCommands::Template => {
            println!("{}", PRD_TEMPLATE);
            Ok(())
        }
    }
}

// ─── Fetch & Format ────────────────────────────────────────────────────────

struct FormattedPrd {
    title: String,
    content: String,
    /// Raw markdown from HTML conversion, before section remapping.
    /// Used by the reviewer to check against original wiki structure.
    raw_md: String,
}

fn fetch_and_format(page_id: &str, insecure: bool) -> Result<FormattedPrd, JiraError> {
    let cfg = Config::load()?;
    cfg.validate()?;
    let client = Client::new(&cfg, insecure);

    let page = wiki_api::get_page(&client, page_id, "body.view,version,space")?;

    let base = page.links.as_ref().map(|l| l.base.as_str()).unwrap_or("");
    let raw_md = page
        .body
        .as_ref()
        .and_then(|b| b.view.as_ref())
        .map(|v| {
            let md = confluence::convert_to_markdown(&v.value, base);
            confluence::download_images(&md, page_id, &client)
        })
        .unwrap_or_default();

    let sections = parse_into_sections(&raw_md, &page.title);

    let mut prd = String::new();

    // Header
    prd.push_str(&format!("# PRD: {}\n\n", page.title));

    // Metadata
    prd.push_str("## Metadata\n\n");
    prd.push_str(&format!("- **Page ID:** {}\n", page.id));
    if let Some(ref s) = page.space {
        prd.push_str(&format!("- **Space:** {}\n", s.key));
    }
    if let Some(ref v) = page.version {
        prd.push_str(&format!("- **Version:** {}\n", v.number));
        if let Some(ref by) = v.by {
            prd.push_str(&format!("- **Last Updated By:** {}\n", by.display_name));
        }
    }
    if let Some(ref l) = page.links {
        if !l.base.is_empty() && !l.web_ui.is_empty() {
            prd.push_str(&format!("- **Source:** {}{}\n", l.base, l.web_ui));
        }
    }
    prd.push('\n');

    // Overview
    prd.push_str("## Overview\n\n");
    if let Some(overview) = sections.overview {
        prd.push_str(&overview);
    } else {
        prd.push_str(&format!(
            "This document describes the requirements for **{}**.\n",
            page.title
        ));
    }
    prd.push_str("\n\n");

    if let Some(scope) = sections.scope {
        prd.push_str("## Scope\n\n");
        prd.push_str(&scope);
        prd.push_str("\n\n");
    }

    if !sections.requirements.is_empty() {
        prd.push_str("## Requirements\n\n");
        for req in &sections.requirements {
            if req.title.is_empty() {
                prd.push_str(&req.content);
            } else {
                prd.push_str(&format!("### {}\n\n", req.title));
                prd.push_str(&req.content);
            }
            prd.push_str("\n\n");
        }
    }

    if !sections.acceptance_criteria_raw.is_empty() || !sections.acceptance_criteria.is_empty() {
        prd.push_str("## Acceptance Criteria\n\n");
        for raw in &sections.acceptance_criteria_raw {
            prd.push_str(raw);
            prd.push_str("\n\n");
        }
        for (i, ac) in sections.acceptance_criteria.iter().enumerate() {
            prd.push_str(&format!("{}. {}\n", i + 1, ac));
        }
        prd.push('\n');
    }

    if let Some(notes) = sections.notes {
        prd.push_str("## Notes\n\n");
        prd.push_str(&notes);
        prd.push_str("\n\n");
    }

    if !sections.remaining.is_empty() {
        prd.push_str("## Additional Details\n\n");
        prd.push_str(&sections.remaining);
        prd.push('\n');
    }

    let re_blank = regex::Regex::new(r"\n{3,}").unwrap();
    let prd = re_blank.replace_all(&prd, "\n\n").to_string();

    Ok(FormattedPrd {
        title: page.title,
        content: prd.trim().to_string(),
        raw_md,
    })
}

fn fetch_raw(page_id: &str, insecure: bool) -> Result<FormattedPrd, JiraError> {
    let cfg = Config::load()?;
    cfg.validate()?;
    let client = Client::new(&cfg, insecure);

    let page = wiki_api::get_page(&client, page_id, "body.view,version,space")?;

    let raw_html = page
        .body
        .as_ref()
        .and_then(|b| b.view.as_ref())
        .map(|v| v.value.clone())
        .unwrap_or_default();

    let base = page.links.as_ref().map(|l| l.base.as_str()).unwrap_or("");
    let text = confluence::convert_to_lightweight_markdown(&raw_html, base);

    let mut content = String::new();

    content.push_str(&format!("PRD: {}\n\n", page.title));
    content.push_str(&format!("Page ID: {}\n", page.id));
    if let Some(ref s) = page.space {
        content.push_str(&format!("Space: {}\n", s.key));
    }
    if let Some(ref v) = page.version {
        content.push_str(&format!("Version: {}\n", v.number));
        if let Some(ref by) = v.by {
            content.push_str(&format!("Last Updated By: {}\n", by.display_name));
        }
    }
    if let Some(ref l) = page.links {
        if !l.base.is_empty() && !l.web_ui.is_empty() {
            content.push_str(&format!("Source: {}{}\n", l.base, l.web_ui));
        }
    }
    content.push_str("\n---\n\n");
    content.push_str(&text);

    Ok(FormattedPrd {
        title: page.title,
        content,
        raw_md: String::new(),
    })
}

// ─── Quality Review (Unified 14-Section Framework) ─────────────────────────

fn do_review(
    page_id: &str,
    title: &str,
    prd_content: &str,
    raw_md: &str,
    post_comment: bool,
    json_output: bool,
    strict: bool,
    insecure: bool,
) -> Result<(), JiraError> {
    let findings = review_prd(prd_content, raw_md, strict);

    let total_deduction: u32 = findings.iter().map(|f| f.points).sum();
    let score = 100u32.saturating_sub(total_deduction);
    let approved = score >= APPROVAL_THRESHOLD;

    let missing_count = findings.iter().filter(|f| matches!(f.severity, Severity::Missing)).count();
    let incomplete_count = findings.iter().filter(|f| matches!(f.severity, Severity::Incomplete)).count();
    let suggestion_count = findings.iter().filter(|f| matches!(f.severity, Severity::Suggestion)).count();

    if json_output {
        print_review_json(page_id, title, score, approved, &findings);
    } else {
        print_review_console(score, approved, &findings, missing_count, incomplete_count, suggestion_count);
    }

    let html = build_review_html(title, score, approved, &findings, missing_count, incomplete_count, suggestion_count);

    if post_comment {
        let cfg = Config::load()?;
        cfg.validate()?;
        let client = Client::new(&cfg, insecure);

        match wiki_api::add_comment(&client, page_id, &html) {
            Ok(()) => {
                eprintln!("\n  [posted] Review comment added to wiki page {}", page_id);
            }
            Err(e) => {
                eprintln!("\n  [warning] Failed to post review comment: {}", e);
            }
        }
    } else if !json_output {
        eprintln!("\n  Use --comment to post these findings to the wiki page");
    }

    Ok(())
}

fn findings_to_review_output(page_id: &str, title: &str, score: u32, approved: bool, findings: &[Finding]) -> ReviewOutput {
    let decision = if approved { "APPROVED" } else { "NEEDS_REVISION" };
    let finding_outputs: Vec<FindingOutput> = findings.iter().map(|f| FindingOutput {
        severity: match f.severity {
            Severity::Missing => "missing",
            Severity::Incomplete => "incomplete",
            Severity::Suggestion => "suggestion",
        }.to_string(),
        section: f.section.clone(),
        points: f.points,
        message: f.message.clone(),
    }).collect();
    ReviewOutput {
        page_id: page_id.to_string(),
        title: title.to_string(),
        score,
        threshold: APPROVAL_THRESHOLD,
        decision: decision.to_string(),
        findings: finding_outputs,
    }
}

fn print_review_json(page_id: &str, title: &str, score: u32, approved: bool, findings: &[Finding]) {
    let output = findings_to_review_output(page_id, title, score, approved, findings);
    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
}

fn print_review_console(
    score: u32,
    approved: bool,
    findings: &[Finding],
    missing_count: usize,
    incomplete_count: usize,
    suggestion_count: usize,
) {
    println!("\n---\n");
    println!("## PRD Quality Review (14-Section Framework)\n");

    if findings.is_empty() {
        println!("No issues found.\n");
    } else {
        for (i, finding) in findings.iter().enumerate() {
            let icon = match finding.severity {
                Severity::Missing => "🔴",
                Severity::Incomplete => "🟡",
                Severity::Suggestion => "🔵",
            };
            println!(
                "{}. {} **{}** (-{} pts) — {}",
                i + 1, icon, finding.section, finding.points, finding.message
            );
        }
        println!();
    }

    let status_icon = if approved { "✅" } else { "❌" };
    let status_text = if approved { "APPROVED" } else { "NEEDS REVISION" };
    println!("**Score: {}/100** {} {}", score, status_icon, status_text);
    if !approved {
        if missing_count > 0 {
            println!("Required: {}/100 to approve. Fix 🔴 Missing items first.", APPROVAL_THRESHOLD);
        } else {
            println!("Required: {}/100 to approve. Address 🟡 Incomplete items.", APPROVAL_THRESHOLD);
        }
    }
    println!("\nSummary: {} missing, {} incomplete, {} suggestions", missing_count, incomplete_count, suggestion_count);
}

fn build_review_html(
    title: &str,
    score: u32,
    approved: bool,
    findings: &[Finding],
    missing_count: usize,
    incomplete_count: usize,
    suggestion_count: usize,
) -> String {
    let status_text = if approved { "APPROVED" } else { "NEEDS REVISION" };
    let badge_color = if approved { "#36B37E" } else { "#FF5630" };

    let mut html = String::new();
    html.push_str(&format!("<h2>PRD Review: {}</h2>", title));
    html.push_str("<p>");
    html.push_str("<strong>Reviewer:</strong> Engineering (AI-assisted)<br/>");
    html.push_str(&format!("<strong>Framework:</strong> 14-Section Standard (threshold: {}/100)<br/>", APPROVAL_THRESHOLD));
    html.push_str(&format!(
        "<strong>Score:</strong> <span style=\"background-color: {}; color: white; padding: 4px 12px; border-radius: 4px; font-weight: bold;\">{}/100 — {}</span>",
        badge_color, score, status_text
    ));
    html.push_str("</p><hr/>");

    html.push_str("<h3>Section Checklist</h3>");
    html.push_str("<table><tr><th>#</th><th>Section</th><th>Status</th><th>Points</th><th>Notes</th></tr>");

    for spec in PRD_SECTIONS {
        let finding = findings.iter().find(|f| f.section == spec.name);
        let (status, points, notes) = match finding {
            Some(f) => {
                let s = match f.severity {
                    Severity::Missing => "MISSING",
                    Severity::Incomplete => "Incomplete",
                    Severity::Suggestion => "Suggestion",
                };
                (s, format!("-{}", f.points), f.message.clone())
            }
            None => ("OK", "0".to_string(), String::new()),
        };
        html.push_str(&format!(
            "<tr><td>{}</td><td><strong>{}</strong></td><td>{}</td><td>{}</td><td>{}</td></tr>",
            spec.id, spec.name, status, points, notes
        ));
    }
    html.push_str("</table><hr/>");

    if missing_count > 0 || incomplete_count > 0 {
        html.push_str("<h3>Action Items</h3>");
        html.push_str("<table><tr><th>Priority</th><th>Section</th><th>Action</th></tr>");
        for f in findings {
            let priority = match f.severity {
                Severity::Missing => "P0 — Blocker",
                Severity::Incomplete => "P1 — Important",
                Severity::Suggestion => "P2 — Nice to have",
            };
            html.push_str(&format!(
                "<tr><td>{}</td><td><strong>{}</strong></td><td>{}</td></tr>",
                priority, f.section, f.message
            ));
        }
        html.push_str("</table><hr/>");
    }

    html.push_str(&format!(
        "<p><strong>Summary:</strong> {} missing, {} incomplete, {} suggestions</p>",
        missing_count, incomplete_count, suggestion_count
    ));
    html.push_str(&format!(
        "<p><strong>Approval threshold:</strong> {}/100. Current score: <strong>{}/100</strong></p>",
        APPROVAL_THRESHOLD, score
    ));

    html
}

enum Severity {
    Missing,
    Incomplete,
    Suggestion,
}

struct Finding {
    severity: Severity,
    section: String,
    points: u32,
    message: String,
}

/// Review PRD quality using the unified 14-section framework.
/// Checks both the final formatted content and the raw markdown.
fn review_prd(content: &str, raw_md: &str, strict: bool) -> Vec<Finding> {
    let mut findings = Vec::new();
    let content_lower = content.to_lowercase();
    let raw_lower = raw_md.to_lowercase();

    let has = |pattern: &str| -> bool {
        content_lower.contains(pattern) || raw_lower.contains(pattern)
    };

    // ─── 1. Metadata ───────────────────────────────────────────────────────
    let has_metadata = has("document status") || has("document owner") || has("version")
        || (has("designer") && (has("figma") || has("status")));
    let has_metadata_complete = has_metadata
        && has("document status") && has("document owner");
    if !has_metadata {
        findings.push(Finding {
            severity: Severity::Missing,
            points: if strict { 5 } else { 3 },
            section: "Metadata".to_string(),
            message: "No metadata table found. Add: Document Status, Owner, Designer, Figma link, Version, Urgency.".to_string(),
        });
    } else if !has_metadata_complete {
        findings.push(Finding {
            severity: Severity::Incomplete,
            points: 2,
            section: "Metadata".to_string(),
            message: "Metadata incomplete. Ensure Document Status and Document Owner are specified.".to_string(),
        });
    }

    // ─── 2. Background & Problem Statement ─────────────────────────────────
    let has_background = has("background") || has("problem statement")
        || has("## problem") || has("### problem");
    if !has_background {
        findings.push(Finding {
            severity: Severity::Missing,
            points: 10,
            section: "Background & Problem Statement".to_string(),
            message: "No background section found. Add a section explaining WHY this feature is needed and what user/business problem it solves.".to_string(),
        });
    }

    // ─── 3. Objectives ─────────────────────────────────────────────────────
    let has_objective = has("objective") || has("## goal") || has("### goal");
    if !has_objective {
        findings.push(Finding {
            severity: Severity::Missing,
            points: 10,
            section: "Objectives".to_string(),
            message: "No objectives section found. Add specific, measurable goals this feature should achieve.".to_string(),
        });
    }

    // ─── 4. Scope ──────────────────────────────────────────────────────────
    let has_scope = has("## scope") || has("### scope") || has("# scope")
        || has("scope requirement") || has("in scope") || has("out of scope")
        || has("in-scope") || has("out-of-scope");
    if !has_scope {
        findings.push(Finding {
            severity: if strict { Severity::Missing } else { Severity::Incomplete },
            points: if strict { 8 } else { 3 },
            section: "Scope".to_string(),
            message: "No scope section found. Add In Scope / Out of Scope to prevent scope creep and set clear boundaries for engineering.".to_string(),
        });
    }

    // ─── 5. General Rules ──────────────────────────────────────────────────
    let has_rules = has("general rule") || has("## constraint") || has("universal rule")
        || has("# general rules") || has("## rule") || has("### rule");
    if !has_rules {
        findings.push(Finding {
            severity: Severity::Incomplete,
            points: if strict { 5 } else { 2 },
            section: "General Rules".to_string(),
            message: "No general rules section. Add performance requirements, data freshness, platform constraints.".to_string(),
        });
    }

    // ─── 6. Acceptance Criteria ────────────────────────────────────────────
    let has_acceptance = has("acceptance criteria") || has("## acceptance");
    if !has_acceptance {
        findings.push(Finding {
            severity: Severity::Missing,
            points: 15,
            section: "Acceptance Criteria".to_string(),
            message: "No acceptance criteria found. Add testable conditions (Given/When/Then) that define when each feature is done.".to_string(),
        });
    } else {
        let has_testable = has("given") && has("when") && has("then");
        let has_numbered = content.lines().any(|l| {
            let t = l.trim();
            t.starts_with("1.") || t.starts_with("- [")
        }) || raw_md.lines().any(|l| {
            let t = l.trim();
            t.starts_with("1.") || t.starts_with("- [")
        });
        if !has_testable && !has_numbered {
            findings.push(Finding {
                severity: Severity::Incomplete,
                points: 5,
                section: "Acceptance Criteria".to_string(),
                message: "Acceptance criteria exist but may not be structured as testable conditions. Use Given/When/Then or numbered checklist.".to_string(),
            });
        }
    }

    // ─── 7. Feature Sections ───────────────────────────────────────────────
    let has_features = has("## enhancement") || has("### enhancement")
        || has("# fe requirement") || has("## fe requirement")
        || has("## compose") || has("## question")
        || has("### layout") || has("### rules")
        || has("scope requirements:") || has("**scope requirements**")
        || has("# scope requirement")
        || has("**location**:");
    if !has_features {
        findings.push(Finding {
            severity: Severity::Missing,
            points: 15,
            section: "Feature Sections".to_string(),
            message: "No feature/enhancement sections found. Add detailed specs per feature with Layout, Rules, Data & Update, Edge Cases.".to_string(),
        });
    } else if strict {
        let has_layout = has("### layout") || has("| layout");
        let has_data_update = has("data & update") || has("data source") || has("api endpoint")
            || has("update behavior") || has("update frequency");
        if !has_layout || !has_data_update {
            let mut missing_subs = Vec::new();
            if !has_layout { missing_subs.push("Layout"); }
            if !has_data_update { missing_subs.push("Data & Update Behavior"); }
            findings.push(Finding {
                severity: Severity::Incomplete,
                points: 5,
                section: "Feature Sections".to_string(),
                message: format!("Feature sections missing sub-sections: {}. Each feature needs Layout, Rules, Data & Update, Edge Cases.", missing_subs.join(", ")),
            });
        }
    }

    // ─── 8. Design Reference ───────────────────────────────────────────────
    let has_figma = has("figma.com") || has("figma link") || has("figma design");
    let has_design_images = has("| design |") || has("| design|") || has("|design|")
        || (has(".png") && has("figure"));
    if !has_figma && !has_design_images {
        findings.push(Finding {
            severity: Severity::Missing,
            points: 8,
            section: "Design Reference".to_string(),
            message: "No design reference found. Add a Figma link or embed design images with figure labels.".to_string(),
        });
    } else if !has_figma && has_design_images {
        findings.push(Finding {
            severity: Severity::Suggestion,
            points: if strict { 3 } else { 2 },
            section: "Design Reference".to_string(),
            message: "Design images found but no Figma link. Add a Figma URL for developers to inspect spacing, colors, and specs.".to_string(),
        });
    }

    // ─── 9. User Flows ─────────────────────────────────────────────────────
    let has_user_flow = has("user flow") || has("user journey") || has("entry point")
        || has("happy path") || has("## flow") || has("behavior flow")
        || (has("stage") && has("interaction"));
    if !has_user_flow {
        findings.push(Finding {
            severity: if strict { Severity::Missing } else { Severity::Incomplete },
            points: if strict { 8 } else { 5 },
            section: "User Flows".to_string(),
            message: "No user flow section found. Add entry points, happy path, and failure/edge paths.".to_string(),
        });
    }

    // ─── 10. Technical Docs / API Contract ─────────────────────────────────
    let has_tech_docs = has("api contract") || has("api endpoint") || has("sequence diagram")
        || has("## technical") || has("### technical") || has("## api")
        || has("## be requirement") || has("# be requirement")
        || has("backend spec") || has("data model") || has("deeplink")
        || has("rest api") || has("websocket");
    if !has_tech_docs {
        findings.push(Finding {
            severity: if strict { Severity::Missing } else { Severity::Incomplete },
            points: if strict { 8 } else { 3 },
            section: "Technical Docs / API Contract".to_string(),
            message: "No technical documentation found. Add API contracts, data models, or backend specs for engineering integration.".to_string(),
        });
    }

    // ─── 11. Event Tracking ────────────────────────────────────────────────
    let has_events = has("event tracking") || has("analytics") || has("## event")
        || has("| event") || has("_viewed") || has("_clicked");
    if !has_events {
        findings.push(Finding {
            severity: Severity::Incomplete,
            points: 3,
            section: "Event Tracking".to_string(),
            message: "No event tracking section. Add analytics events table (Event Name / Trigger / Properties).".to_string(),
        });
    }

    // ─── 12. LCMP Keys ────────────────────────────────────────────────────
    let has_l10n = has("lcmp") || has("localization") || has("multilanguage")
        || has("| key | en |") || has("| key | id |");
    if !has_l10n {
        findings.push(Finding {
            severity: Severity::Incomplete,
            points: 3,
            section: "LCMP Keys".to_string(),
            message: "No LCMP keys or localization table. Add keys for all user-facing strings (Key / ID / EN / ZH).".to_string(),
        });
    }

    // ─── 13. Success Metrics ───────────────────────────────────────────────
    let has_success = has("success definition") || has("success metric")
        || has("## success") || has("### success") || has("kpi");
    if !has_success {
        findings.push(Finding {
            severity: if strict { Severity::Missing } else { Severity::Incomplete },
            points: 5,
            section: "Success Metrics".to_string(),
            message: "No success metrics found. Add primary and secondary KPIs to measure feature impact.".to_string(),
        });
    }

    // ─── 14. Related PRDs ──────────────────────────────────────────────────
    if !has("related prd") && !has("## related") {
        findings.push(Finding {
            severity: Severity::Suggestion,
            points: 2,
            section: "Related PRDs".to_string(),
            message: "No related PRDs section. Consider linking dependent or related specifications.".to_string(),
        });
    }

    // ─── Cross-cutting: Vague Language ─────────────────────────────────────
    let vague_phrases = [
        "improve the", "better user experience", "make it easier",
        "enhance the", "optimize the", "as needed", "if necessary",
        "should be good", "nice to have", "tbd", "to be decided",
    ];
    for phrase in &vague_phrases {
        if content_lower.contains(phrase) {
            findings.push(Finding {
                severity: Severity::Incomplete,
                points: 3,
                section: "Requirements Clarity".to_string(),
                message: format!(
                    "Vague language found: \"{}\". Replace with specific, measurable requirements.",
                    phrase
                ),
            });
            break;
        }
    }

    // ─── Cross-cutting: Figure-Reference Validation ────────────────────────
    let figure_count = count_pattern(&content_lower, "figure x.") + count_pattern(&raw_lower, "figure x.");
    let rule_ref_count = count_pattern(&content_lower, "figure x.") + count_pattern(&content_lower, "show figure");
    if figure_count > 0 && rule_ref_count == 0 {
        findings.push(Finding {
            severity: Severity::Suggestion,
            points: 2,
            section: "Design-Rule Linkage".to_string(),
            message: format!("Found {} design figures but rules don't reference them. Link each figure to its corresponding rule.", figure_count),
        });
    }

    findings
}

fn count_pattern(text: &str, pattern: &str) -> usize {
    text.matches(pattern).count()
}

// ─── PRD Template ──────────────────────────────────────────────────────────

const PRD_TEMPLATE: &str = r#"# PRD: T-XXXXXX — Feature Title

<!-- PRD Template v3 — 11-Section Compact Standard -->
<!-- All 11 sections are REQUIRED. If a section is not applicable, keep -->
<!-- the heading and write a brief explanation (e.g. "No backend changes"). -->
<!-- Approval threshold: 95/100. See scoring guide at the end. -->

## 1. Metadata

| Field | Value |
|-------|-------|
| **Document Status** | DRAFT / IN REVIEW / APPROVED |
| **Document Owner** | PM Name |
| **Business Owner** | Stakeholder Name |
| **Designer** | Designer Name |
| **Figma** | [Design Link](https://www.figma.com/...) |
| **MRD** | [MRD Link](https://your-wiki.example.com/...) |
| **Confluence** | https://your-wiki.example.com/pages/viewpage.action?pageId=XXXXX |
| **Request Type** | New Feature / Improvement / Bug Fix |
| **Version** | 1.0 — DD Month YYYY |
| **Urgency** | Low / Medium / High / Critical |

### Change Log

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | DD Month YYYY | PM Name | Initial draft |

---

## 2. Executive Summary (TL;DR)

Two-to-four sentence summary. What is this feature, who is it for, why does it
matter now, and what is the primary outcome we expect? A reader should be able
to understand the whole PRD from just this section.

---

## 3. Background & Problem Statement

Why does this feature need to exist? What user/business problem does it solve?
What is the current state and what happens if we don't build it? Cite data,
research, or incidents where possible.

---

## 4. Objectives & Success Metrics

### Objectives
- Specific, measurable goal 1 (verifiable with data)
- Specific, measurable goal 2

### Success Metrics
| Metric | Target | Measurement |
|--------|--------|-------------|
| Primary KPI (e.g. adoption rate) | > X% | 14 days after launch |
| Secondary KPI | > Y% | 30 days after launch |
| Guardrail (e.g. crash-free rate) | > 99.5% | continuous |

---

## 5. Scope

### In Scope
- Feature A — description
- Feature B — description

### Out of Scope
- Feature C — reason (e.g., deferred to Phase 2)
- Feature D — reason

---

## 6. User Stories

<!-- Format: As a <persona>, I want to <action> so that <benefit> -->

1. As a **first-time user**, I want to see an onboarding tutorial so that I understand the feature.
2. As a **returning user**, I want to skip the tutorial so that I can act quickly.
3. As a **user on slow network**, I want cached data so that the feature still works offline.

---

## 7. Functional Requirements

### Feature 1: Feature Name

### Layout
- Where does this appear in the UI?
- Visual hierarchy and component structure

### Rules
- Business logic and conditions
- State transitions (deterministic, no ambiguity)

### Data & Update Behavior
1. Data source: API endpoint / WebSocket event
2. Request/Response shape (key fields)
3. Update frequency: real-time / polling / on-demand
4. Cache strategy

### Edge Cases

| State | Behavior | Design |
|-------|----------|--------|
| Loading | Show skeleton/shimmer | figure X.N |
| Loaded | Show data | figure X.N |
| Empty | Show empty state illustration | figure X.N |
| Error | Show error with retry button | figure X.N |
| Offline | Show cached data or offline banner | figure X.N |

---

### Feature 2: Feature Name

(Repeat the same structure: Layout, Rules, Data & Update, Edge Cases)

---

## 8. Design Reference

- **Figma**: [Full Design File](https://www.figma.com/...)
- All screens and states must have a figure reference (figure X.N)
- Each figure must be referenced by at least one rule in § 7

---

## 9. User Flows / User Journey

### Primary Journey
```
Entry point → Step 1 → Step 2 → Step 3 → Success
                ↓
           (Error / Edge path) → Recovery
```

| # | Step | User Action | System Response | Design |
|---|------|-------------|-----------------|--------|
| 1 | Entry | User opens feature | Show main screen | figure X.1 |
| 2 | Primary action | User taps CTA | Submit request | figure X.2 |
| 3 | Success | — | Confirmation state | figure X.3 |

### Edge Paths
- Exit without saving → changes discarded, no confirmation prompt
- Offline → show cached data, sync on reconnect
- Error → show error state with retry button

---

## 10. Acceptance Criteria

<!-- Given/When/Then. Each criterion must be testable. -->

| # | Scenario | Given | When | Then |
|---|----------|-------|------|------|
| 1 | Happy path | User is logged in | User performs primary action | Success result shown |
| 2 | Error state | API returns 500 | User performs action | Error toast with retry |
| 3 | Offline | Device has no network | User opens feature | Cached data shown |
| 4 | Empty state | No data exists | User opens feature | Empty illustration shown |

---

## 11. Risks & Open Questions

### Risks
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| API latency spike | Medium | High | Client-side cache + backoff |
| User confusion with new flow | High | Medium | Onboarding tutorial |

### Open Questions
<!-- If none: "No open questions — all resolved" -->

| # | Question | Owner | Due |
|---|----------|-------|-----|
| 1 | Tablet layout support in v1? | PM + Design | YYYY-MM-DD |
| 2 | SLA on new endpoint? | BE Lead | YYYY-MM-DD |

---

<!-- SCORING GUIDE (for reviewers — 11-Section Standard, deductions sum to 100):

  Section                                    Missing  Incomplete
  1.  Metadata                                  -4       -2
  2.  TL;DR                                     -5       -2
  3.  Background & Problem                     -10       -3
  4.  Objectives & Success Metrics             -12       -4
  5.  Scope (In/Out)                            -8       -3
  6.  User Stories                              -7       -2
  7.  Functional Requirements                  -18       -6
  8.  Design Reference                          -8       -3
  9.  User Flows / User Journey                 -8       -3
  10. Acceptance Criteria                      -15       -5
  11. Risks & Open Questions                    -5       -2

  Vague language penalty: -3 points (per offence, cap -9)
  Approval threshold: 95/100
  Sections with valid "N/A" note: 0 deduction
-->
"#;

// ─── Section Parser ─────────────────────────────────────────────────────────

struct ParsedSections {
    overview: Option<String>,
    scope: Option<String>,
    requirements: Vec<SectionBlock>,
    acceptance_criteria: Vec<String>,
    /// Raw acceptance criteria content preserved as-is (for pipe tables)
    acceptance_criteria_raw: Vec<String>,
    notes: Option<String>,
    remaining: String,
}

struct SectionBlock {
    title: String,
    content: String,
}

fn parse_into_sections(md: &str, page_title: &str) -> ParsedSections {
    let mut result = ParsedSections {
        overview: None,
        scope: None,
        requirements: Vec::new(),
        acceptance_criteria: Vec::new(),
        acceptance_criteria_raw: Vec::new(),
        notes: None,
        remaining: String::new(),
    };

    let cleaned = clean_raw_md(md, page_title);
    let blocks = split_by_headings(&cleaned);

    if blocks.is_empty() && !cleaned.trim().is_empty() {
        result.remaining = cleaned.trim().to_string();
        return result;
    }

    for (heading, content) in &blocks {
        let h_lower = heading.to_lowercase();
        let content = content.trim().to_string();
        if content.is_empty() {
            continue;
        }

        if heading.is_empty()
            || h_lower.contains("general")
            || h_lower.contains("overview")
            || h_lower.contains("background")
        {
            if result.overview.is_none() {
                result.overview = Some(content);
            } else {
                result.requirements.push(SectionBlock {
                    title: heading.clone(),
                    content,
                });
            }
        } else if h_lower.contains("scope") {
            let existing = result.scope.take().unwrap_or_default();
            result.scope = Some(if existing.is_empty() {
                content
            } else {
                format!("{}\n\n{}", existing, content)
            });
        } else if h_lower.contains("acceptance") || h_lower.contains("criteria") {
            // Check if content contains pipe tables — preserve them as-is
            let has_pipe_table = content.lines().any(|l| {
                let t = l.trim();
                t.starts_with('|') && t.ends_with('|') && t.matches('|').count() >= 3
            });
            if has_pipe_table {
                result.acceptance_criteria_raw.push(content.clone());
            } else {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                        result.acceptance_criteria.push(trimmed[2..].to_string());
                    } else if !trimmed.is_empty()
                        && !trimmed.starts_with('#')
                        && trimmed.len() > 3
                    {
                        result.acceptance_criteria.push(trimmed.to_string());
                    }
                }
            }
        } else if h_lower.contains("note")
            || h_lower.contains("dependency")
            || h_lower.contains("constraint")
            || h_lower.contains("limitation")
        {
            let existing = result.notes.take().unwrap_or_default();
            result.notes = Some(if existing.is_empty() {
                content
            } else {
                format!("{}\n\n{}", existing, content)
            });
        } else {
            result.requirements.push(SectionBlock {
                title: heading.clone(),
                content,
            });
        }
    }

    if result.overview.is_none() && !result.requirements.is_empty() {
        let first = &result.requirements[0];
        let line_count = first.content.lines().count();
        let has_images = first.content.contains("![");
        if line_count <= 5 && !has_images {
            let removed = result.requirements.remove(0);
            result.overview = Some(removed.content);
        }
    }

    result
}

fn clean_raw_md(md: &str, page_title: &str) -> String {
    let lines: Vec<&str> = md.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let title_lower = page_title.to_lowercase();
    let mut seen_headings: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.starts_with("**Hi Fi") || trimmed.starts_with("**Hi-Fi") {
            i += 1;
            continue;
        }

        if is_table_column_header(trimmed) {
            i += 1;
            continue;
        }

        if i + 1 < lines.len() {
            let next = lines[i + 1].trim();
            if next.len() >= 3
                && (next.chars().all(|c| c == '=') || next.chars().all(|c| c == '-'))
            {
                let heading_text = trimmed
                    .replace("**", "")
                    .trim_matches('#')
                    .trim()
                    .to_string();
                let heading_key = heading_text.to_lowercase();

                if heading_key.contains(&title_lower) || title_lower.contains(&heading_key) {
                    if seen_headings.contains(&heading_key) {
                        i += 2;
                        continue;
                    }
                }
                seen_headings.insert(heading_key);
                let level = if next.starts_with('=') { "##" } else { "###" };
                out.push(format!("{} {}", level, heading_text));
                i += 2;
                continue;
            }
        }

        if trimmed.starts_with('#') {
            let heading_text = trimmed
                .trim_start_matches('#')
                .trim()
                .replace("**", "")
                .trim_matches('#')
                .trim()
                .to_string();
            let heading_key = heading_text.to_lowercase();

            if heading_key.contains(&title_lower) || title_lower.contains(&heading_key) {
                if seen_headings.contains(&heading_key) {
                    i += 1;
                    continue;
                }
            }
            seen_headings.insert(heading_key);
            let level = trimmed.split_whitespace().next().unwrap_or("#");
            out.push(format!("{} {}", level, heading_text));
            i += 1;
            continue;
        }

        out.push(line.to_string());
        i += 1;
    }

    out.join("\n")
}

fn is_table_column_header(s: &str) -> bool {
    let column_headers = [
        "Previous", "Current", "Requirement", "Hi Fi Design",
        "Design Notes", "What Jie Li Have", "Change from Jieli",
        "Jie Li Jiao Yi Bao", "Tuntun Design",
    ];
    column_headers.iter().any(|h| s == *h)
}

fn split_by_headings(md: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut current_heading = String::new();
    let mut current_content = Vec::new();

    for line in md.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let content = current_content.join("\n").trim().to_string();
            if !content.is_empty() || !current_heading.is_empty() {
                blocks.push((current_heading.clone(), content));
            }
            current_heading = trimmed
                .trim_start_matches('#')
                .trim()
                .trim_end_matches('#')
                .trim()
                .replace("**", "")
                .to_string();
            current_content.clear();
        } else {
            current_content.push(line.to_string());
        }
    }

    let content = current_content.join("\n").trim().to_string();
    if !content.is_empty() || !current_heading.is_empty() {
        blocks.push((current_heading, content));
    }

    blocks
}

fn save_prd(title: &str, content: &str) {
    let project_root = match find_project_root() {
        Some(root) => root,
        None => return,
    };

    let prd_dir = project_root.join(".tuntun").join("prd");
    if std::fs::create_dir_all(&prd_dir).is_err() {
        return;
    }

    let safe_title = title
        .replace('/', "-")
        .replace('\\', "-")
        .replace(':', "-")
        .replace('*', "")
        .replace('?', "")
        .replace('"', "")
        .replace('<', "")
        .replace('>', "")
        .replace('|', "-")
        .trim()
        .to_string();

    let filename = format!("{}.md", safe_title);
    let filepath = prd_dir.join(&filename);

    match std::fs::write(&filepath, content) {
        Ok(()) => eprintln!("  [saved] {}", filepath.display()),
        Err(e) => eprintln!("  [warning] Failed to save PRD: {}", e),
    }
}

fn save_prd_raw(title: &str, content: &str) {
    let project_root = match find_project_root() {
        Some(root) => root,
        None => return,
    };

    let prd_dir = project_root.join(".tuntun").join("prd");
    if std::fs::create_dir_all(&prd_dir).is_err() {
        return;
    }

    let safe_title = title
        .replace('/', "-")
        .replace('\\', "-")
        .replace(':', "-")
        .replace('*', "")
        .replace('?', "")
        .replace('"', "")
        .replace('<', "")
        .replace('>', "")
        .replace('|', "-")
        .trim()
        .to_string();

    let filename = format!("{}.raw.md", safe_title);
    let filepath = prd_dir.join(&filename);

    match std::fs::write(&filepath, content) {
        Ok(()) => eprintln!("  [saved] {}", filepath.display()),
        Err(e) => eprintln!("  [warning] Failed to save PRD: {}", e),
    }
}

fn find_project_root() -> Option<std::path::PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join(".tuntun").exists() || dir.join(".claude").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}
