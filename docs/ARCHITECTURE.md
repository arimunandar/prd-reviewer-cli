# Architecture

Technical reference for engineering reviewers. Read
`PRODUCT_OVERVIEW.md` first for context.

---

## High-level design

```
┌────────────────────────────────────────────────────────────────────┐
│                     prd-reviewer (Rust CLI)                        │
│                                                                    │
│  Subcommands:                                                      │
│  ├── prd fetch    → pulls wiki page, downloads attachments         │
│  ├── prd rules    → emits 11-section standard (md / JSON)          │
│  ├── prd workflow → emits 7-step review workflow                   │
│  ├── prd template → emits blank PRD skeleton                       │
│  ├── jira ...     → Confluence + Jira REST API ops                 │
│  ├── figma ...    → Figma API ops                                  │
│  ├── init         → installs .claude/ skill + agent into project   │
│  └── update       → self-update from GitHub                        │
└────────────────────────┬───────────────────────────────────────────┘
                         │  fetches PRD + emits rules
                         ▼
┌────────────────────────────────────────────────────────────────────┐
│                   Claude Code (user's machine)                     │
│                                                                    │
│  ┌──────────────────────┐      ┌──────────────────────┐            │
│  │ /prd-reviewer skill  │      │ @prd-reviewer agent  │            │
│  │ (interactive)        │      │ (autonomous)         │            │
│  │                      │      │                      │            │
│  │ • Read PRD + images  │      │ • Same flow          │            │
│  │ • Load rules         │      │ • No human in loop   │            │
│  │ • Judge by meaning   │      │   except confirms    │            │
│  │ • AskUserQuestion    │      │ • Posts to wiki at   │            │
│  │ • Score + report     │      │   the end            │            │
│  │ • Offer wiki post    │      │                      │            │
│  └──────────────────────┘      └──────────────────────┘            │
└────────────────────────────────────────────────────────────────────┘
```

---

## Key design decisions

### 1. CLI does NOT judge

Earlier versions (v1.0 and before) used keyword matching in Rust to
decide if a section was present — `content.contains("tl;dr") ||
content.contains("executive summary")`. This produced false
negatives whenever PMs used different phrasing or put content in
visual tables.

**v1.1+ decision:** the CLI is purely a data provider. It emits
rules and PRD content; the AI (via the skill / agent) applies
meaning-based judgment.

Benefit: 500 fewer lines of brittle Rust code, and dramatically
better review accuracy.

### 2. Rules live in one place

`src/commands/prd.rs::PRD_SECTIONS` is the single source of truth
for section names, weights, and check criteria. It's emitted in two
formats:

- Markdown (`prd rules`) — for humans
- JSON (`prd rules --json`) — for the AI to consume programmatically

The same weights appear in the skill and agent docs — but those are
documentation, not enforcement. The CLI is authoritative.

### 3. Images are first-class input

When the CLI fetches a wiki page, it walks every `![alt](url)` image
reference, downloads the image to
`<project>/.prd-reviewer/images/<page_id>/<filename>`, and rewrites
the markdown link to the local path.

The skill then uses the `Read` tool on each image — making the
design, mockups, and screenshots part of the review context. PMs
encode half the rules in screenshots (empty state, copy placement,
padlock states); ignoring them guaranteed false positives.

### 4. AskUserQuestion over guessing

In every mode (Review / Generate / Adjust), the AI must call
`AskUserQuestion` whenever a finding is ambiguous — not guess.
Typical triggers:

- LCMP section absent → "backend-only feature?"
- Objectives exist, KPIs absent → "KPIs in a linked doc?"
- Out-of-Scope missing → "intentionally everything is in scope?"
- Thin Acceptance Criteria → "how should error state behave?"

This prevents false verdicts. It also creates a **record** — answers
become PRD amendments or Open Questions logged for later.

### 5. Engineer FAQ is always on

Structural review catches structural gaps ("no Acceptance Criteria").
Engineers block on operational gaps ("where is state stored?").
The FAQ is a separate required section that surfaces the operational
questions the review framework itself doesn't measure.

---

## Code layout

```
src/
├── main.rs                      # CLI entry point, clap subcommand router
├── config.rs                    # ~/.prd-reviewer.yaml parsing
├── commands/
│   ├── mod.rs
│   ├── prd.rs                   # fetch / rules / workflow / template (the core)
│   ├── jira.rs                  # Confluence + Jira REST operations
│   ├── figma.rs                 # Figma API operations
│   ├── init.rs                  # installs .claude/ skill + agent
│   └── update.rs                # self-update from GitHub
├── jira/
│   ├── client.rs                # shared HTTP client with auth
│   ├── api/
│   │   └── wiki_api.rs          # Confluence endpoints
│   └── confluence/
│       ├── html.rs              # HTML → Markdown conversion
│       ├── images.rs            # attachment download + cache
│       └── jira_wiki.rs         # Jira wiki markup parser
├── figma/                       # Figma API client + subcommands
└── templates/
    └── mod.rs                   # CLAUDE.md + skill + agent text (shipped by `init`)

.claude/
├── skills/prd-reviewer/
│   └── SKILL.md                 # /prd-reviewer skill (installed per project)
└── agents/
    └── prd-reviewer.md          # @prd-reviewer agent (installed per project)

docs/
├── PRODUCT_OVERVIEW.md          # management-facing pitch + roadmap
├── SAMPLE_REVIEW.md             # real-world sample output
└── ARCHITECTURE.md              # this file
```

---

## Dependencies

All pulled from crates.io:

| Crate | Use |
|---|---|
| `clap` | CLI arg parsing |
| `ureq` | HTTP client |
| `serde` / `serde_json` / `serde_yaml` | Config + API (de)serialization |
| `rustls` | TLS for self-signed internal hosts |
| `html2md` | Confluence HTML → Markdown |
| `regex` | Markdown post-processing |
| `sha2` / `hmac` | Cache-filename hashing + API signing |
| `comfy-table` | Table rendering in CLI output |
| `base64` | API auth encoding |

No database, no server, no external services besides the user's own
Jira / Wiki / Figma and the Claude Code runtime.

---

## Release & update flow

- Each versioned commit bumps `Cargo.toml` and is pushed to `main`.
- `prd-reviewer update` clones / pulls the GitHub repo into
  `~/.prd-reviewer/cli/` and runs `cargo install --path .`.
- Skill + agent markdown is re-synced into any project that already
  has `.claude/skills/prd-reviewer/` or a `CLAUDE.md` with the
  prd-reviewer marker.

---

## Security notes

- `~/.prd-reviewer.yaml` is written with `chmod 600` by the installer.
- All API calls use Bearer tokens; no plaintext secrets in logs.
- PRDs and downloaded images are cached in `<project>/.prd-reviewer/`,
  which is in the default `.gitignore` — PRD content never enters
  version control.
- The CLI never transmits PRD content anywhere except the user's own
  Wiki (for commenting) and the local Claude Code client.
