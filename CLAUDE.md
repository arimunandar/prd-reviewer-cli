<!-- prd-reviewer -->
# PRD Co-Pilot — Review · Generate · Adjust

This project is the `prd-reviewer` CLI: a Rust tool + Claude Code skill/agent
pair that helps product teams review, generate, and adjust Product Requirement
Documents against an 11-section compact standard.

## Three Modes

The `/prd-reviewer` skill auto-detects the mode from the user's prompt:

- **Review** — user shares a wiki URL / page ID → fetch, score (Layer 1),
  deep semantic audit (Layer 2), produce a scored report (optionally posted
  to wiki).
- **Generate** — user gives a brief or idea → interview via `AskUserQuestion`,
  then draft a full PRD that passes the 11-section standard.
- **Adjust** — user has an existing PRD with gaps → diagnose, ask for the
  missing context, rewrite in place, re-score.

For autonomous end-to-end work (fetch → interview → draft/review → wiki
post), spawn `@prd-reviewer` instead.

**Approval threshold: 95/100.**

## 11-Section Compact Standard

| # | Section | Missing | Incomplete |
|---|---------|---------|------------|
| 1 | Metadata | -4 | -2 |
| 2 | TL;DR (Executive Summary) | -5 | -2 |
| 3 | Background & Problem Statement | -10 | -3 |
| 4 | Objectives & Success Metrics | -12 | -4 |
| 5 | Scope (In/Out) | -8 | -3 |
| 6 | User Stories | -7 | -2 |
| 7 | Functional Requirements (Features) | -18 | -6 |
| 8 | Design Reference (Figma) | -8 | -3 |
| 9 | User Flows / User Journey | -8 | -3 |
| 10 | Acceptance Criteria (Given/When/Then) | -15 | -5 |
| 11 | Risks & Open Questions | -5 | -2 |

Deductions sum to 100. Sections can be marked N/A with a brief explanation
for 0 deduction. Full weights, automation-readiness criteria, and report
format live in `.claude/skills/prd-reviewer/SKILL.md`.

## Components

- **CLI** (`src/`): `prd`, `jira`, `figma`, `init`, `update` commands
- **Skill** (`.claude/skills/prd-reviewer/SKILL.md`): 3-mode workflow with
  `AskUserQuestion` interviews for Generate/Adjust
- **Agent** (`.claude/agents/prd-reviewer.md`): autonomous end-to-end co-pilot

## Quick Reference

```bash
# PRD tools
prd-reviewer prd fetch <PAGE_ID> --raw           # Raw markdown
prd-reviewer prd fetch <PAGE_ID>                 # Structured markdown
prd-reviewer prd score <PAGE_ID>                 # JSON score (Layer 1)
prd-reviewer prd review <PAGE_ID>                # Structural review
prd-reviewer prd review <PAGE_ID> --comment      # Post review to wiki
prd-reviewer prd template                        # PRD template v3 (11 sections)

# Confluence Wiki / Jira
prd-reviewer jira wiki page view <ID> --insecure
prd-reviewer jira wiki page search --title "<q>" --insecure
prd-reviewer jira wiki page comment <ID> --file <html> --insecure
prd-reviewer jira issue issue view <KEY> --insecure

# Figma (inspect PRD design references)
prd-reviewer figma url '<URL>'                   # Inspect + 2x screenshot
prd-reviewer figma screenshot '<URL>'            # Screenshot only
prd-reviewer figma url '<URL>' --tree            # ASCII hierarchy
prd-reviewer figma view '<URL>'                  # Table overview
```

## Configuration

All credentials are stored in `~/.prd-reviewer.yaml` (permissions 0600):

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

Run `install.sh` to set all credentials at once.

## Build

```bash
cargo build --release
cargo install --path .
prd-reviewer init                                # Sync skill + agent to a project
```
