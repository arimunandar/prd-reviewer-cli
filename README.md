# prd-reviewer

**PRD Co-Pilot for product teams — Review · Generate · Adjust.**

`prd-reviewer` is a Rust CLI + Claude Code skill/agent that helps product
teams review existing PRDs, generate new PRDs from briefs, and adjust
PRDs that have gaps — all against an 11-section compact standard with a
95/100 approval gate.

> 📚 **New here?** Start with
> [docs/PRODUCT_OVERVIEW.md](docs/PRODUCT_OVERVIEW.md) — the full
> product pitch, value story, and adoption plan. For a real-world
> sample review output, see [docs/SAMPLE_REVIEW.md](docs/SAMPLE_REVIEW.md).
> For technical architecture, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Features

- **Three modes** auto-detected from user intent:
  - **Review** — score an existing PRD against the 11-section standard
  - **Generate** — draft a new PRD from a brief, interviewing the PM via
    `AskUserQuestion` only when needed
  - **Adjust** — diagnose gaps in an existing PRD, ask targeted questions,
    rewrite in place, re-score
- **Fetch PRDs** from Confluence Wiki as clean markdown
- **Inspect Figma designs** referenced by PRDs
- **Post reviews** back to the wiki as HTML comments

## Install

One-liner — clones, builds, and prompts for credentials:

```bash
git clone https://github.com/arimunandar/prd-reviewer-cli.git
cd prd-reviewer-cli
./install.sh
```

The installer will:

1. Check for Rust (installs via `rustup` if missing)
2. Build & install the `prd-reviewer` binary via `cargo install`
3. Prompt for credentials and write them to `~/.prd-reviewer.yaml` (chmod 600)

### What you'll be asked for

| Service | Required? | Where to get it |
|---------|-----------|-----------------|
| Jira Base URL | Yes | e.g. `https://your-jira.example.com/rest/api/2` |
| Jira Access Token | Yes | Jira → Profile → Personal Access Tokens |
| Wiki Base URL | Yes | e.g. `https://your-wiki.example.com/rest/api/content` |
| Wiki Access Token | Yes | Confluence → Profile → Personal Access Tokens |
| Figma Token | Optional | Figma → Settings → Security → Personal access tokens |

URL format is validated (must start with `http://` or `https://`). Secrets are
read silently. If you re-run `install.sh` with an existing config, it will
ask before overwriting.

You can also edit `~/.prd-reviewer.yaml` directly at any time.

## Quick Start

The CLI is a **data provider**. It fetches PRDs and emits the 11-section rules
+ workflow. The AI (via the `/prd-reviewer` skill or `@prd-reviewer` agent)
owns the judgment and interviews the PM via `AskUserQuestion` when a section
is ambiguous.

```bash
# Initialize a project (installs skill + agent + CLAUDE.md section)
prd-reviewer init

# Fetch a PRD for the AI to review
prd-reviewer prd fetch 12345 --raw

# Load the 11-section rules
prd-reviewer prd rules                # markdown
prd-reviewer prd rules --json         # machine-readable (for the AI)

# Read the review workflow the AI follows
prd-reviewer prd workflow

# Canonical PRD template (for Generate mode)
prd-reviewer prd template
```

## Using the Skill inside Claude Code

```
/prd-reviewer <wiki_page_id | URL | feature brief>
```

The skill auto-routes to the right mode:

- `/prd-reviewer 12345` → **Review** the wiki page
- `/prd-reviewer write a PRD for two-factor login` → **Generate** from brief
- `/prd-reviewer improve the PRD at ./drafts/login.md` → **Adjust** existing

For fully autonomous end-to-end work (fetch → interview → draft/review →
wiki post), use the agent:

```
@prd-reviewer <anything>
```

## 11-Section Compact Standard

| # | Section | Max Deduction |
|---|---------|---------------|
| 1 | Metadata | -4 |
| 2 | TL;DR (Executive Summary) | -5 |
| 3 | Background & Problem Statement | -10 |
| 4 | Objectives & Success Metrics | -12 |
| 5 | Scope (In/Out) | -8 |
| 6 | User Stories | -7 |
| 7 | Functional Requirements | -18 |
| 8 | Design Reference | -8 |
| 9 | User Flows / User Journey | -8 |
| 10 | Acceptance Criteria | -15 |
| 11 | Risks & Open Questions | -5 |

**Approval threshold: 95/100.** Deductions sum to 100. Sections can be
marked N/A with a brief explanation for 0 deduction.

## Configuration

Credentials live in `~/.prd-reviewer.yaml` (permissions 0600):

```yaml
jira:
  access_token: "Bearer token"
  base_url: "https://your-jira.example.com/rest/api/2"
wiki:
  access_token: "Bearer token"
  base_url: "https://your-wiki.example.com/rest/api/content"
figma:
  personal_token: "Figma Personal Access Token"
```

## CLI Reference

```
prd-reviewer prd <fetch|rules|workflow|template> [args]
prd-reviewer jira <issue|wiki> ...
prd-reviewer figma <url|view|screenshot|node|file|variable|comment|...> ...
prd-reviewer init [--path <dir>]
prd-reviewer update
prd-reviewer version
```

Run any subcommand with `--help` for details.

## License

MIT.
