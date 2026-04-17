use clap::Subcommand;
use serde::Serialize;

use crate::jira::api::wiki_api;
use crate::jira::client::Client;
use crate::jira::config::Config;
use crate::jira::confluence;
use crate::jira::error::JiraError;

// ─── 11-Section Compact Standard ───────────────────────────────────────────
// The CLI is a data provider: it emits these rules and the review workflow.
// The /prd-reviewer skill and @prd-reviewer agent own all judgment — they
// read the fetched PRD + these rules, reason by meaning (not keywords),
// interview the PM via AskUserQuestion when a section is ambiguous, compute
// the score, and produce the report.
//
// Deductions sum to 100. Approval threshold: 95/100.

struct SectionSpec {
    id: u8,
    name: &'static str,
    weight_missing: u32,
    weight_incomplete: u32,
    check: &'static str,
}

const PRD_SECTIONS: &[SectionSpec] = &[
    SectionSpec { id: 1,  name: "Metadata",                     weight_missing: 4,  weight_incomplete: 2,
        check: "Document Status, Owner, Designer, Figma link, Version, Changelog, Urgency" },
    SectionSpec { id: 2,  name: "TL;DR",                        weight_missing: 5,  weight_incomplete: 2,
        check: "2–4 sentence executive summary: what, who, why now, primary outcome" },
    SectionSpec { id: 3,  name: "Background & Problem",         weight_missing: 10, weight_incomplete: 3,
        check: "Why this exists — user/business problem, current state, impact if not built" },
    SectionSpec { id: 4,  name: "Objectives & Success Metrics", weight_missing: 12, weight_incomplete: 4,
        check: "Measurable goals AND KPI targets with measurement window" },
    SectionSpec { id: 5,  name: "Scope (In/Out)",               weight_missing: 8,  weight_incomplete: 3,
        check: "Explicit In-Scope and Out-of-Scope lists to prevent scope creep" },
    SectionSpec { id: 6,  name: "User Stories",                 weight_missing: 7,  weight_incomplete: 2,
        check: "≥ 3 stories: \"As a <persona>, I want X so that Y\"" },
    SectionSpec { id: 7,  name: "Functional Requirements",      weight_missing: 18, weight_incomplete: 6,
        check: "Per feature: Layout, Rules, Data & Update Behavior, Edge Cases (Loading/Loaded/Empty/Error/Offline)" },
    SectionSpec { id: 8,  name: "Design Reference",             weight_missing: 8,  weight_incomplete: 3,
        check: "Figma link; every screen/state labeled figure X.N; each figure referenced by ≥ 1 rule" },
    SectionSpec { id: 9,  name: "User Flows / Journey",         weight_missing: 8,  weight_incomplete: 3,
        check: "Entry points, primary journey, edge paths with recovery" },
    SectionSpec { id: 10, name: "Acceptance Criteria",          weight_missing: 15, weight_incomplete: 5,
        check: "Given/When/Then covering happy path, error, empty, offline" },
    SectionSpec { id: 11, name: "Risks & Open Questions",       weight_missing: 5,  weight_incomplete: 2,
        check: "Risks table (likelihood/impact/mitigation) + open questions with owner + due date" },
];

const APPROVAL_THRESHOLD: u32 = 95;
const STANDARD_VERSION: &str = "v3";

const AUTOMATION_READINESS: &[&str] = &[
    "Deterministic rules — unambiguous, measurable conditions (no \"improve\" / \"better\" / \"nice\")",
    "Complete data contracts — every data-driven feature names endpoint / payload / event",
    "State coverage — Loading · Loaded · Empty · Error · Offline",
    "Figure-to-rule mapping — every figure X.N is referenced by at least one rule",
    "Testable acceptance criteria — Given/When/Then or numbered verifiable conditions",
    "LCMP completeness — every user-facing string has a localization key",
];

// ─── Serializable Rules Output ─────────────────────────────────────────────

#[derive(Serialize)]
struct RulesJson {
    standard_version: &'static str,
    threshold: u32,
    sections: Vec<SectionJson>,
    automation_readiness: Vec<&'static str>,
    scoring: ScoringJson,
}

#[derive(Serialize)]
struct SectionJson {
    id: u8,
    name: &'static str,
    weight_missing: u32,
    weight_incomplete: u32,
    check: &'static str,
}

#[derive(Serialize)]
struct ScoringJson {
    formula: &'static str,
    missing: &'static str,
    incomplete: &'static str,
    ok: &'static str,
    na_with_explanation: &'static str,
}

#[derive(Subcommand)]
pub enum PrdCommands {
    /// Fetch a wiki PRD page as markdown (feed this to the AI reviewer)
    Fetch {
        /// Wiki page ID
        page_id: String,
        /// Output raw lightweight markdown (preserves original structure)
        #[arg(long)]
        raw: bool,
        /// Skip TLS verification
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Emit the 11-section review rules (markdown by default, --json for machine-readable)
    Rules {
        /// Emit as JSON instead of markdown
        #[arg(long)]
        json: bool,
    },
    /// Emit the PRD review workflow the AI should follow
    Workflow,
    /// Output the canonical PRD template (v3 — 11 sections)
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
        PrdCommands::Fetch { page_id, raw, insecure } => {
            if raw {
                let prd = fetch_raw(&page_id, insecure)?;
                println!("{}", prd.content);
                save_prd_raw(&prd.title, &prd.content);
                return Ok(());
            }
            let prd = fetch_and_format(&page_id, insecure)?;
            println!("{}", prd.content);
            save_prd(&prd.title, &prd.content);
            Ok(())
        }
        PrdCommands::Rules { json } => {
            if json {
                print_rules_json();
            } else {
                print_rules_markdown();
            }
            Ok(())
        }
        PrdCommands::Workflow => {
            print!("{}", PRD_WORKFLOW);
            Ok(())
        }
        PrdCommands::Template => {
            println!("{}", PRD_TEMPLATE);
            Ok(())
        }
    }
}

// ─── Rules + Workflow Emitters ─────────────────────────────────────────────

fn print_rules_markdown() {
    println!("# PRD Review Rules — 11-Section Compact Standard ({})", STANDARD_VERSION);
    println!();
    println!("Approval threshold: **{}/100**. Deductions sum to 100.", APPROVAL_THRESHOLD);
    println!();
    println!("## Sections");
    println!();
    println!("| # | Section | Missing | Incomplete | What to check |");
    println!("|---|---------|---------|------------|---------------|");
    for s in PRD_SECTIONS {
        println!(
            "| {} | {} | -{} | -{} | {} |",
            s.id, s.name, s.weight_missing, s.weight_incomplete, s.check
        );
    }
    println!();
    println!("## Scoring");
    println!();
    println!("- **score = 100 − Σ(deductions)**");
    println!("- **Missing** — section absent or unrecognizable → full Missing weight");
    println!("- **Incomplete** — present but gaps → Incomplete weight");
    println!("- **OK** — complete and automation-ready → 0");
    println!("- **N/A with explanation** — section doesn't apply and PRD says so → 0");
    println!();
    println!("## Automation-Readiness Criteria");
    println!();
    println!("Applied especially to Functional Requirements and Acceptance Criteria:");
    println!();
    for c in AUTOMATION_READINESS {
        println!("- {}", c);
    }
    println!();
    println!("## How to use");
    println!();
    println!("1. Fetch the PRD: `prd-reviewer prd fetch <PAGE_ID> --raw`");
    println!("2. Apply these rules by meaning, not keyword matching");
    println!("3. When a section's status is ambiguous, ask the PM via `AskUserQuestion`");
    println!("4. Never invent data — log unknowns as Open Questions");
    println!("5. See `prd-reviewer prd workflow` for the full step-by-step");
}

fn print_rules_json() {
    let payload = RulesJson {
        standard_version: STANDARD_VERSION,
        threshold: APPROVAL_THRESHOLD,
        sections: PRD_SECTIONS
            .iter()
            .map(|s| SectionJson {
                id: s.id,
                name: s.name,
                weight_missing: s.weight_missing,
                weight_incomplete: s.weight_incomplete,
                check: s.check,
            })
            .collect(),
        automation_readiness: AUTOMATION_READINESS.to_vec(),
        scoring: ScoringJson {
            formula: "score = 100 - sum(deductions)",
            missing: "section absent or unrecognizable — apply full Missing weight",
            incomplete: "section present but has gaps — apply Incomplete weight",
            ok: "section complete and automation-ready — 0",
            na_with_explanation: "section doesn't apply and PRD says so — 0",
        },
    };
    println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
}

// ─── Fetch & Format ────────────────────────────────────────────────────────

struct FormattedPrd {
    title: String,
    content: String,
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
    let text = confluence::download_images(&text, page_id, &client);

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
    })
}


// ─── PRD Review Workflow ──────────────────────────────────────────────────

const PRD_WORKFLOW: &str = r#"# PRD Review Workflow — 7 Steps

The CLI is a data provider; the AI (via the `/prd-reviewer` skill or
`@prd-reviewer` agent) applies judgment. Follow these steps.

## Step 1 — Fetch the PRD (text + attachments)

```bash
prd-reviewer prd fetch <PAGE_ID> --raw
```

- Read the saved file at `.prd-reviewer/prd/<title>.raw.md`.
- **Also read every image/attachment** — the fetcher downloads them to
  `.prd-reviewer/images/<page_id>/` and rewrites the inline links.
  Open each image; designs and screenshots carry rules the text omits.

## Step 2 — Load the rules

```bash
prd-reviewer prd rules --json     # machine-readable
prd-reviewer prd rules            # human-readable markdown
```

These emit the 11-section standard, weights, and automation-readiness criteria.

## Step 3 — Section-by-section review (AI judgment)

For each of the 11 sections, decide by **meaning**, not keyword matching:

1. **Present?** — does a section covering this exist, even under a different heading?
2. **Complete?** — does it cover everything the rules expect?
3. **Automation-ready?** — can an engineer act on it without asking clarifying questions?

Classify each as **OK** / **Incomplete** / **Missing** / **N/A-with-note**.

## Step 4 — Interview the PM to clarify findings

Whenever a finding is ambiguous, borderline, or depends on PM intent that
isn't written down, ask via `AskUserQuestion` **before** locking the verdict.
Do NOT guess. Examples:

- **LCMP absent?** → "Is this a backend-only change with no user-facing strings?"
  Options: [ "Yes — mark N/A", "No — needs LCMP keys" ]
- **Objectives exist but no KPIs?** → "Are the success metrics in a linked doc?"
  Options: [ "Yes — linked (OK)", "No — add here (Incomplete)" ]
- **Out-of-Scope missing?** → "Is this intentionally omitted?"
  Options: [ "Yes — everything is in scope", "No — needs explicit list" ]

Ask one question per ambiguous section, batched in a single round where possible.
Skip the interview for clearly missing sections (e.g. no Acceptance Criteria at
all is unambiguously missing — no need to ask).

## Step 5 — Compute score

```
score = 100 − Σ(deductions)
```

Use the Missing / Incomplete weights from the rules. N/A-with-note = 0.

## Step 6 — Generate report

Produce a markdown report with:

- **Score** (XX/100) and decision (APPROVED ≥ 95 / NEEDS REVISION)
- **Section Checklist** — ALL 11 sections, even when OK
- **Blockers (P0)** — Missing or critically Incomplete
- **Quality Issues (P1)** — fixable gaps
- **Suggestions (P2)** — nice-to-haves
- **Engineer FAQ** — REQUIRED, always. 6 categories (Data & Persistence,
  State & Concurrency, Error & Offline, Platform & Device, Integration
  Contracts, Observability & Rollout). Tag each question ANSWERED /
  PARTIAL / OPEN. Lets the PM resolve opens before the assessment
  meeting so engineers can focus on true edge cases.
- **Strengths** — always include 3–5 positives (reviews must be balanced)
- **Action Items** — priority / item / owner

## Step 7 — (Optional) post to wiki

Ask the user first via `AskUserQuestion`. If yes:

```bash
prd-reviewer jira wiki page comment <PAGE_ID> --file <review.html> --insecure
```

All 11 sections must appear in the checklist HTML. The Engineer FAQ table
is also required in the HTML. No emojis. No invented data.
"#;

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

    let prd_dir = project_root.join(".prd-reviewer").join("prd");
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

    let prd_dir = project_root.join(".prd-reviewer").join("prd");
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
        if dir.join(".prd-reviewer").exists() || dir.join(".claude").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}
