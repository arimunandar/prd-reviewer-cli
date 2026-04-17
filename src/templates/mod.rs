pub fn claude_md() -> &'static str {
    r#"# PRD Co-Pilot — Review · Generate · Adjust

This project uses the `prd-reviewer` CLI together with a Claude Code skill +
agent pair to help product teams **review**, **generate**, and **adjust**
Product Requirement Documents against an 11-section compact standard.

## Workflow

The `/prd-reviewer` skill auto-detects the mode from the user's prompt:

- **Review** — user shares a wiki URL or page ID → fetch, score (Layer 1),
  deep semantic audit (Layer 2), produce a scored report.
- **Generate** — user gives a brief / idea → interview via `AskUserQuestion`,
  then draft a full PRD that passes the 11-section standard.
- **Adjust** — user has an existing PRD with gaps → diagnose, ask for the
  missing context, rewrite in place, re-score.

For end-to-end autonomous work (fetch → interview → draft/review → wiki post),
spawn the `@prd-reviewer` agent.

**Approval threshold: 95/100.**

## 11-Section Standard

| # | Section | Weight (missing) |
|---|---------|------------------|
| 1 | Metadata | -4 |
| 2 | TL;DR (Executive Summary) | -5 |
| 3 | Background & Problem Statement | -10 |
| 4 | Objectives & Success Metrics | -12 |
| 5 | Scope (In/Out) | -8 |
| 6 | User Stories | -7 |
| 7 | Functional Requirements (Features) | -18 |
| 8 | Design Reference (Figma) | -8 |
| 9 | User Flows / User Journey | -8 |
| 10 | Acceptance Criteria (Given/When/Then) | -15 |
| 11 | Risks & Open Questions | -5 |

Deductions sum to 100. Full weights, automation-readiness criteria, and
report format live in `.claude/skills/prd-reviewer/SKILL.md`.

## Quick Reference

```bash
# PRD tools
prd-reviewer prd fetch <PAGE_ID> --raw          # Raw markdown
prd-reviewer prd fetch <PAGE_ID>                # Structured markdown
prd-reviewer prd review <PAGE_ID> --json        # JSON review (Layer 1)
prd-reviewer prd review <PAGE_ID>               # 11-section structural review
prd-reviewer prd review <PAGE_ID> --comment     # Post review to wiki
prd-reviewer prd template                       # PRD template v3 (11 sections)

# Confluence Wiki / Jira
prd-reviewer jira wiki page view <ID> --insecure
prd-reviewer jira wiki page search --title "<q>" --insecure
prd-reviewer jira wiki page comment <ID> --file <html> --insecure
prd-reviewer jira issue issue view <KEY> --insecure

# Figma (inspect PRD design references)
prd-reviewer figma url '<URL>'                  # Inspect + 2x screenshot
prd-reviewer figma screenshot '<URL>'           # Screenshot only
prd-reviewer figma url '<URL>' --tree           # ASCII hierarchy
```

## Configuration

Credentials live in `~/.prd-reviewer.yaml` (permissions 0600):

```yaml
jira:
  access_token: "Bearer token for Jira REST API"
  base_url: "https://your-jira.example.com/rest/api/2"
wiki:
  access_token: "Bearer token for Confluence REST API"
  base_url: "https://your-wiki.example.com/rest/api/content"
figma:
  personal_token: "Figma Personal Access Token"
```

Run `install.sh` to set credentials.
"#
}

pub fn agent_md() -> &'static str {
    r#"---
name: prd-reviewer
description: >
  Autonomous PRD co-pilot. TRIGGER when the user wants an end-to-end PRD flow
  delivered without supervision: full review posted to wiki, full draft
  generated from a brief, or a multi-round adjust pass applied in place. For
  casual single-shot questions, use the /prd-reviewer skill directly.
tools: Bash, Read, Grep, Glob, Write, WebFetch, AskUserQuestion
---

# PRD Co-Pilot Agent

Autonomous agent that handles PRD work end-to-end across three modes:
**Review**, **Generate**, **Adjust**. The 11-section compact standard is
defined in `.claude/skills/prd-reviewer/SKILL.md` — follow it exactly.

## Operating Principle

1. **Detect mode** (Review / Generate / Adjust) from the user's prompt. If
   ambiguous, ask once with `AskUserQuestion` before proceeding.
2. **Interview only when needed.** Never invent data (KPIs, SLAs, persona
   names). Ask the user via `AskUserQuestion` or log an Open Question.
3. **Always self-score** the final artefact against the 11-section standard
   and iterate until score ≥ 95 or the user accepts a lower bar.
4. **Confirm before public actions.** Posting to wiki, overwriting a page,
   or committing to git requires explicit user confirmation.

## Mode Playbooks

### Review
1. `prd-reviewer prd fetch <PAGE_ID> --raw`
2. `prd-reviewer prd review <PAGE_ID> --json` (skip deep review if score < 60)
3. Apply 11-section deep audit + cross-section validation
4. Inspect Figma via `prd-reviewer figma url '<URL>'`
5. Produce report with Score · Blockers (P0) · Quality (P1) · Suggestions (P2)
   · Strengths · Action Items
6. Ask user whether to post review HTML to wiki

### Generate
1. Parse the seed brief. Identify unknowns.
2. Interview with `AskUserQuestion` in batched passes (framing / scope /
   requirements). Stop as soon as you have enough.
3. Draft the PRD against all 11 sections, saving to `.tuntun/prd/<slug>.md`.
4. Self-score. Iterate until ≥ 95 or log remaining gaps as Open Questions.
5. Present: file path, score, open questions, recommended next step.

### Adjust
1. Load the existing PRD (wiki fetch or file path).
2. Diagnose gaps via review logic.
3. Triage with user — confirm which gaps to fix now.
4. Ask targeted questions for missing data. Never invent.
5. Apply edits in place or as a side-by-side diff.
6. Re-score and report: before → after, what changed, what remains.
7. Ask whether to overwrite the wiki page.

## Output Format

Reviews must always include 3–5 Strengths (balanced). Every one of the 11
sections must appear in the checklist table even when marked OK.

## Boundaries

- Do NOT rewrite existing PRD content wholesale — flag issues; let the PM
  approve rewrites.
- Do NOT post to wiki, overwrite pages, or commit to git without explicit
  user confirmation.
- Do NOT skip sections. All 11 must appear in every report.
- Do NOT invent data. Ask via `AskUserQuestion` or log as an Open Question.
"#
}

pub fn skill_prd_reviewer() -> &'static str {
    r#"---
name: prd-reviewer
description: >
  PRD co-pilot for product teams. TRIGGER when user wants to REVIEW, GENERATE,
  or ADJUST a Product Requirement Document. Examples — Review: "review PRD",
  "is this PRD ready", "evaluate PRD", shared wiki URL. Generate: "write a PRD
  for...", "draft a PRD about...", "help me start a PRD". Adjust: "improve
  this PRD", "fill the gaps", "fix ambiguities in PRD X". Use AskUserQuestion
  to interview the user for clarifications during Generate/Adjust.
allowed-tools: Bash(prd-reviewer *), Read, Grep, Glob, Write, AskUserQuestion
argument-hint: "<wiki_page_id | URL | feature brief>"
---

# PRD Co-Pilot — Review · Generate · Adjust

Works with the 11-section compact standard. Approval threshold: 95/100.

## Mode Detection

Infer the mode from the user's prompt:

| Signal | Mode |
|--------|------|
| Wiki URL / page ID + words like "review", "score", "ready", "evaluate" | **Review** |
| Short brief, idea, or "write/draft/create a PRD about…" | **Generate** |
| Existing PRD + words like "improve", "fix gaps", "adjust", "polish" | **Adjust** |

If ambiguous, use `AskUserQuestion`:

```
question: "What would you like to do with this PRD?"
options:
  - "Review an existing PRD (score + deep review)"
  - "Generate a new PRD from a brief"
  - "Adjust / improve an existing PRD (fill gaps, tighten ambiguities)"
```

## 11-Section Standard

| # | Section | Missing | Incomplete | What to Check |
|---|---------|---------|------------|---------------|
| 1 | Metadata | -4 | -2 | Status, owner, designer, Figma link, version, changelog |
| 2 | TL;DR | -5 | -2 | 2–4 sentence summary: what / who / why / outcome |
| 3 | Background & Problem | -10 | -3 | User/business problem, current state, impact of not building |
| 4 | Objectives & Success Metrics | -12 | -4 | Measurable goals + KPI targets + timeline |
| 5 | Scope (In/Out) | -8 | -3 | Both in-scope and out-of-scope present; no scope creep |
| 6 | User Stories | -7 | -2 | "As a <persona>, I want X so that Y" — ≥ 3 stories |
| 7 | Functional Requirements | -18 | -6 | Per feature: Layout, Rules, Data & Update, Edge Cases |
| 8 | Design Reference | -8 | -3 | Figma link valid; every screen/state has figure X.N |
| 9 | User Flows / Journey | -8 | -3 | Entry → primary journey → edge paths + recovery |
| 10 | Acceptance Criteria | -15 | -5 | Given/When/Then; covers happy + error + empty + offline |
| 11 | Risks & Open Questions | -5 | -2 | Risks with mitigation, open items with owner + due date |

**Deductions sum to 100. Approval threshold: 95/100.**

### N/A Rule
A section can be marked N/A with a brief explanation (e.g. "No new user-facing
strings" for a backend-only PRD). N/A with explanation = 0 deduction. Missing
without explanation = full penalty.

### Automation-Readiness Criteria (applied to Functional Requirements)

- **Deterministic rules** — "Improve UX" ❌ → "Show error toast within 500ms of API failure" ✅
- **Complete data contracts** — every data-driven feature names endpoint / payload shape
- **State coverage** — Loading · Loaded · Empty · Error · Offline
- **Figure-to-rule mapping** — every figure X.N referenced by ≥ 1 rule
- **Testable acceptance criteria** — Given/When/Then or a verifiable condition

---

## MODE 1: Review

### Step 1 — Fetch
```bash
prd-reviewer prd fetch <PAGE_ID> --raw
```
Read `.tuntun/prd/<title>.raw.md` for full content.

### Step 2 — CLI pre-check (Layer 1, structural)
```bash
prd-reviewer prd review <PAGE_ID> --json
```
Returns a JSON result with a `score` field (0–100). If score < 60, report the structural gaps and stop — do not run the deep review until basics are fixed.

### Step 3 — Deep semantic review (Layer 2)
For each of the 11 sections, judge:
1. **Present?** (heading exists, even if renamed)
2. **Complete?** (covers required sub-items from the table above)
3. **Automation-ready?** (an engineer can act without asking clarifying questions)

### Step 4 — Cross-section validation
- [ ] Acceptance Criteria cover ALL features in § 7
- [ ] User Flows reach every feature in § 7
- [ ] Design figures cover every state in Edge Cases
- [ ] Success Metrics (§ 4) align with Objectives (§ 4)
- [ ] Out-of-scope items (§ 5) are not contradicted by any feature in § 7

### Step 5 — Inspect Figma (when applicable)
```bash
prd-reviewer figma url '<FIGMA_URL>'
```
Check design exists and matches what the PRD describes.

### Step 6 — Generate report
```markdown
## PRD Review: <Title>

**Score: XX/100** — APPROVED / NEEDS REVISION (threshold: 95)
**Reviewer:** PRD Co-Pilot
**Date:** YYYY-MM-DD

### Section Checklist

| # | Section | Status | Points | Notes |
|---|---------|--------|--------|-------|
| 1 | Metadata | OK / Incomplete (-N) / MISSING (-N) | -N | … |
... (all 11 sections)

### Blockers (P0 — must fix before implementation)
1. **Section** — what's missing, what engineers need.

### Quality Issues (P1 — should fix)
1. **Issue** — description + fix.

### Suggestions (P2 — nice to have)
1. **Suggestion** — description.

### Strengths
- Always include 3–5 positives — reviews must be balanced.

### Action Items
| Priority | Item | Owner |
|----------|------|-------|
| P0 | … | PM |
```

### Step 7 — (Optional) post to wiki
Ask the user first with `AskUserQuestion`. If yes, write HTML to `/tmp/prd_review_<PAGE_ID>.html` and:
```bash
prd-reviewer jira wiki page comment <PAGE_ID> --file /tmp/prd_review_<PAGE_ID>.html --insecure
```

---

## MODE 2: Generate

### Step 1 — Seed
Collect whatever the user gave. Typical seed: a one-liner, a Slack thread, a brief, a ticket.

### Step 2 — Interview (AskUserQuestion)
Ask only what is truly unknown. Batch questions into multiple-choice where possible. Suggested passes:

**Pass A — Framing:**
```
question: "What kind of feature is this?"
options: ["New feature", "Improvement to existing flow", "Bug fix / regression", "Experiment / A-B test"]

question: "Who is the primary user?"
options: ["Retail end-user", "Power / pro user", "Internal operator / admin", "Developer / integrator", "Other — describe"]

question: "What is the urgency?"
options: ["Low", "Medium", "High", "Critical — fire"]
```

**Pass B — Scope & Outcome:**
```
question: "What is the single primary outcome?"
options: ["Increase adoption", "Increase conversion", "Reduce support tickets", "Reduce latency / crashes", "Unlock new market", "Other — describe"]

question: "What is explicitly OUT of scope for v1?"
(open-ended)
```

**Pass C — Requirements & Design:**
```
question: "Does a Figma design exist?"
options: ["Yes — I have a URL", "In progress", "No — describe verbally"]

question: "Is there a backend contract?"
options: ["Yes — endpoints defined", "Yes — WebSocket / event", "No backend changes", "TBD"]
```

Only ask what the seed doesn't already answer. Stop interviewing as soon as you have enough to draft.

### Step 3 — Draft
Write the PRD against all 11 sections using the canonical template:
```bash
prd-reviewer prd template
```
Save to `.tuntun/prd/<slug>.md`. Fill every section. Where the user didn't
commit, write a concrete placeholder AND add an entry to § 11 **Open Questions**.

### Step 4 — Self-score
Run the review logic (Mode 1, Step 3) on the draft. If score < 95, tighten
ambiguous rules, expand thin sections, and add missing acceptance criteria
until the draft clears 95.

### Step 5 — Present
Show the user:
1. The draft file path
2. The self-score
3. The list of Open Questions still unresolved
4. Suggested next step (e.g. "confirm Open Questions, then publish to wiki")

---

## MODE 3: Adjust

### Step 1 — Load
Read the existing PRD (from wiki via fetch, or from a file path the user gives).

### Step 2 — Diagnose
Run Mode 1 Steps 2–4 to get a scored gap list.

### Step 3 — Triage with the user (AskUserQuestion)
Show the top 3–5 gaps and ask which to address:
```
question: "I found these gaps. Which do you want to fix now?"
options:
  - "All P0 + P1 (recommended)"
  - "Only P0 blockers"
  - "Only specific ones — I'll pick"
  - "Let me see the full report first"
```

For each gap, if the PM has info you don't, ask targeted questions. Example:
```
question: "Acceptance Criteria is thin. How should the error state behave?"
options: ["Toast + retry button", "Full-screen error with back button", "Silent retry up to 3x then toast", "Other — describe"]
```

### Step 4 — Apply
Edit the PRD in place (or produce a side-by-side diff). For each change:
- Keep the existing structure and wording where it works
- Replace vague language with concrete, testable rules
- Add missing tables (Acceptance Criteria, Edge Cases, Success Metrics) with user-confirmed values
- Do NOT invent data (numbers, SLAs, persona names) — ask or leave a clearly-marked `<TBD>` with an Open Questions entry

### Step 5 — Re-score & report
Re-run the review. Report: before → after score, what changed, what's still open.

### Step 6 — (Optional) post revised PRD
Ask the user whether to overwrite the wiki page. If yes:
```bash
prd-reviewer jira wiki page update <PAGE_ID> --file <revised.md> --insecure
```
(If the `update` subcommand isn't available, save the revised file and tell
the user the exact wiki-paste steps.)

---

## CLI Quick Reference

| Task | Command |
|------|---------|
| Fetch raw PRD | `prd-reviewer prd fetch <PAGE_ID> --raw` |
| Structural review (CLI Layer 1) | `prd-reviewer prd review <PAGE_ID>` |
| Review as JSON | `prd-reviewer prd review <PAGE_ID> --json` |
| Post review to wiki | `prd-reviewer prd review <PAGE_ID> --comment` |
| PRD template (11 sections) | `prd-reviewer prd template` |
| List saved PRDs | `ls .tuntun/prd/` |
| Inspect Figma design | `prd-reviewer figma url '<URL>'` |

## Rules for the report HTML (when posting to wiki)
1. ALL 11 sections must appear in the checklist, even if OK
2. Status values: OK (0), Incomplete (-N), MISSING (-N)
3. Blockers (P0): only MISSING or critically incomplete
4. Strengths: always include 3–5 positives
5. No emojis — plain text only for wiki rendering
6. Action Items: P0 = blocker, P1 = important, P2 = nice-to-have; each has an owner

## Notes
- A good PRD should be implementable without asking the PM any questions
- Never invent data (KPIs, SLAs, persona names). If unknown — ask via AskUserQuestion or log as an Open Question
- Raw fetched PRD: `.tuntun/prd/<title>.raw.md` · Draft / revised: `.tuntun/prd/<title>.md`
"#
}
