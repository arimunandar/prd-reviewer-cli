# PRD Co-Pilot — Product Overview

**Version:** 1.3.0
**Last updated:** 2026-04-17
**Status:** Live — [github.com/arimunandar/prd-reviewer-cli](https://github.com/arimunandar/prd-reviewer-cli)
**Author:** Ari Munandar

---

## 1. Executive Summary

**PRD Co-Pilot** is an AI-powered quality gate for Product Requirement
Documents. It helps product teams **review**, **generate**, and
**adjust** PRDs against an 11-section compact standard — enforcing the
same quality bar every time, and replacing hours of back-and-forth
between PM and engineering with a single scored report.

**One-line pitch:** A PRD goes in. A scored report, an Engineer FAQ,
and a list of action items come out — in under two minutes.

**Who it's for:** Product managers, engineering leads, and designers
who want PRDs that are implementable without clarification meetings.

**What's in the box:**
- A Rust CLI that fetches PRDs from Confluence Wiki, inspects Figma
  designs, and emits the review rules + workflow
- A Claude Code **skill** (`/prd-reviewer`) for interactive work
- A Claude Code **agent** (`@prd-reviewer`) for autonomous, end-to-end
  reviews posted back to the wiki
- An 11-section compact PRD standard (v3) with a 95/100 approval gate

---

## 2. The Problem We're Solving

PRDs are the single highest-leverage document in product delivery —
but in practice they're also the single biggest source of delay,
rework, and friction.

### What we see today

| Symptom | Cost |
|---|---|
| Engineers hit the PRD for the first time during kickoff; first 60–90 minutes are clarification, not estimation | 1–1.5 eng-hours per engineer per feature |
| PMs write PRDs to personal taste — Figma-heavy from one PM, text-heavy from another | Inconsistent quality; reviewers recalibrate every time |
| Design exists only in Figma, screenshots embedded without figure labels; rules don't reference the designs they depend on | Ambiguity → build-first-fix-later |
| "Acceptance Criteria" sections exist but are narrative paragraphs, not testable Given/When/Then | QA writes test cases from scratch; regression risk |
| Non-functional requirements (accessibility, reduced motion, rollout, risks) are skipped silently | Late-stage surprises; last-minute scope cuts |
| Review happens in comments threads on the wiki, with no shared framework | Feedback is subjective; PMs don't know when "done" is done |

### What's at stake

- **Cycle time** — every unclear PRD is a half-day of back-and-forth
- **Quality** — ambiguous PRDs ship features that don't match intent
- **Morale** — engineers feel they're being handed incomplete work;
  PMs feel their writeups are being rewritten in Slack threads

### Root cause

There is no **single, objective, machine-checkable standard** for
"is this PRD ready to build?". PRD Co-Pilot is that standard, plus
the tooling to enforce it automatically.

---

## 3. The Solution — PRD Co-Pilot

A three-part system:

```
┌─────────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│   prd-reviewer CLI  │ ── │ /prd-reviewer    │ ── │ @prd-reviewer    │
│   (data provider)   │    │ Claude skill     │    │ Claude agent     │
│   Rust binary       │    │ (interactive)    │    │ (autonomous)     │
└─────────────────────┘    └──────────────────┘    └──────────────────┘
         │                           │                       │
         │ fetches PRD from wiki     │ reviews by meaning    │ end-to-end
         │ downloads attachments     │ interviews the PM     │ review +
         │ emits 11-section rules    │ scores + reports      │ wiki post
         │ emits review workflow     │ posts to wiki         │
         └───────────────────────────┴───────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │                       │
         ┌──────────────────┐    ┌──────────────────┐
         │ Confluence Wiki  │    │       Figma      │
         │ (PRDs live here) │    │ (designs here)   │
         └──────────────────┘    └──────────────────┘
```

### Three operating modes

| Mode | Trigger | Output |
|---|---|---|
| **Review** | PM shares a wiki URL / page ID → "is this PRD ready?" | Scored 11-section report + Engineer FAQ + action items |
| **Generate** | PM shares a brief / idea → "write a PRD for…" | Full draft passing the 11-section standard, with Open Questions for anything unknown |
| **Adjust** | PM has an existing PRD with gaps → "fix the gaps" | In-place revision, diff-style, re-scored |

**All three modes interview the PM via `AskUserQuestion` when any
finding is ambiguous.** The AI never invents data — KPIs, SLAs,
persona names, edge-case decisions all come from the PM or get logged
as Open Questions.

---

## 4. How It Works

### End-to-end review flow

```
1. PM pastes wiki URL into Claude Code:      /prd-reviewer 76096147
2. Skill fetches PRD + downloads images:     prd-reviewer prd fetch 76096147 --raw
3. AI reads every attached image:            (designs, mockups, storyboards)
4. Skill loads the 11-section rules:         prd-reviewer prd rules --json
5. AI judges each section BY MEANING:        (not keyword matching — "Goals" = "Objectives")
6. AI interviews PM on ambiguous findings:   AskUserQuestion
7. AI scores the PRD and generates report:   Score · Checklist · Blockers · FAQ · Strengths
8. AI asks PM to post to wiki:               prd-reviewer jira wiki page comment …
```

### Why the CLI is a thin data provider

Keyword-based review was the wrong architecture. PMs write PRDs in
natural language — "Executive Summary", "TL;DR", and "Overview" all
mean the same thing. A Rust regex will miss 2 of 3 and flag the
section as missing.

The fix: **the CLI stops judging**. It provides:

- `prd fetch <id>` — pull the PRD text and download every image
- `prd rules` — emit the 11-section standard (markdown or JSON)
- `prd workflow` — emit the 7-step review process
- `prd template` — emit a blank PRD skeleton

The AI reads all of the above, then applies meaning-based judgment.
This is 500 fewer lines of brittle keyword logic, and vastly higher
review accuracy.

---

## 5. The 11-Section Compact Standard

Deductions sum to exactly 100. Approval threshold: **95/100**.

| # | Section | Missing | Incomplete | What "complete" means |
|---|---|---|---|---|
| 1 | **Metadata** | -4 | -2 | Status, Owner, Designer, Figma, Version, Changelog, Urgency |
| 2 | **TL;DR** | -5 | -2 | 2–4 sentence executive summary: what, who, why now, outcome |
| 3 | **Background & Problem** | -10 | -3 | User/business problem + current state + impact if unbuilt |
| 4 | **Objectives & Success Metrics** | -12 | -4 | Measurable goals + KPI targets + measurement window |
| 5 | **Scope (In/Out)** | -8 | -3 | Explicit In-Scope AND Out-of-Scope lists |
| 6 | **User Stories** | -7 | -2 | ≥ 3 stories in "As a <persona>, I want X so that Y" form |
| 7 | **Functional Requirements** | -18 | -6 | Per feature: Layout, Rules, Data & Update, Edge Cases |
| 8 | **Design Reference** | -8 | -3 | Figma link + figure X.N labels referenced by rules |
| 9 | **User Flows / Journey** | -8 | -3 | Entry → primary journey → edge paths with recovery |
| 10 | **Acceptance Criteria** | -15 | -5 | Given/When/Then covering happy, error, empty, offline |
| 11 | **Risks & Open Questions** | -5 | -2 | Risks with mitigation + open items with owner + due date |

**N/A rule:** A section can be explicitly marked N/A with a brief
explanation (e.g. "No user-facing strings" for backend-only PRDs) →
zero deduction. Silent absence = full penalty.

### Automation-readiness criteria

Applied especially to Functional Requirements and Acceptance Criteria:

- **Deterministic rules** — "Improve UX" ❌ → "Show error toast within 500 ms of API failure" ✅
- **Complete data contracts** — every data-driven feature names endpoint, payload, or event
- **State coverage** — Loading · Loaded · Empty · Error · Offline
- **Figure-to-rule mapping** — every design figure (figure X.N) is referenced by at least one rule
- **Testable acceptance criteria** — Given/When/Then or numbered verifiable conditions
- **LCMP completeness** — every user-facing string has a localization key

---

## 6. The Engineer FAQ — the accelerator

Every review includes an **Engineer FAQ** section (new in v1.3.0).
This is the highest-leverage feature in the product — it addresses
the single biggest cost in the current flow: engineers spending 60–90
minutes in assessment meetings clarifying rather than estimating.

### How it works

The AI generates a list of common engineering questions across **six
categories**, and tags each one:

| Status | Meaning |
|---|---|
| ✅ **ANSWERED** | Already in the PRD — reviewer cites the section |
| 🟡 **PARTIAL** | Implied but not explicit — needs tightening |
| ❌ **OPEN** | PM must resolve before the engineering kickoff |

### The six categories

1. **Data & Persistence** — where is state stored, who owns it, reset rule, install/re-install behaviour
2. **State & Concurrency** — same action twice, two devices, app backgrounded mid-flow, race conditions
3. **Error & Offline** — API failure, network drop mid-flow, cached state priority
4. **Platform & Device** — iPad, min iOS version, dark mode, locale, clock/timezone, reduced-motion
5. **Integration Contracts** — endpoint, WebSocket event, payload shape, who calls whom
6. **Observability & Rollout** — logs, analytics events, feature flag, kill-switch, rollback trigger

### Why this matters

Before: PM writes PRD → engineering kickoff → 90 minutes of "what
about X?" questions → estimation is rushed → rework in sprint.

After: PM reviews PRD + FAQ → PM resolves OPEN items solo (10–20
minutes) → engineering kickoff is 30 minutes, focused on edge-case
debate and estimation → clean hand-off.

**Always on, regardless of score.** Even a 96/100 PRD benefits —
"what's the Lottie file-size budget?" is the kind of question no
section template will prompt.

---

## 7. Integration Architecture

### External systems

| System | Used for |
|---|---|
| **Confluence Wiki** | Source of PRD content. CLI fetches HTML, converts to markdown, caches locally, downloads attachments |
| **Figma** | Design-file inspection. CLI can pull screenshots, export assets, list components / styles / variables |
| **Jira** | Ticket metadata, comments, status transitions (used when a PRD links to a Jira story) |

### Configuration

Single file: `~/.prd-reviewer.yaml` (chmod 0600):

```yaml
jira:
  access_token: "Bearer ..."      # Jira REST API token
  base_url: "https://..."         # your Jira instance
wiki:
  access_token: "Bearer ..."      # Confluence REST API token
  base_url: "https://..."         # your Confluence instance
figma:
  personal_token: "figd_..."      # Figma Personal Access Token
```

Installer (`install.sh`) prompts for each and writes the file
automatically. URLs are validated; secrets are read silently; overwrite
confirmation on re-run.

### Data locations

| Path | Purpose |
|---|---|
| `~/.prd-reviewer.yaml` | Credentials |
| `~/.prd-reviewer/cli/` | Cache of the prd-reviewer source repo (used by `prd-reviewer update`) |
| `<project>/.prd-reviewer/prd/` | Fetched PRDs saved here (raw + structured) |
| `<project>/.prd-reviewer/images/<page_id>/` | Downloaded wiki attachments |
| `<project>/.claude/skills/prd-reviewer/SKILL.md` | The Claude Code skill (installed by `prd-reviewer init`) |
| `<project>/.claude/agents/prd-reviewer.md` | The Claude Code agent (installed by `prd-reviewer init`) |

---

## 8. Expected Value — what we measure

### Time saved

| Activity | Before | After | Delta |
|---|---|---|---|
| PRD review (per PRD) | 60–90 min spread across 3–5 Slack exchanges | 2 minutes (CLI + AI) | -85% |
| Engineering kickoff meeting | 90 min (mostly clarifications) | 30 min (edge-case debate only) | -60 min per meeting × N engineers |
| Rework from ambiguous acceptance criteria | 0.5–1 sprint per feature (estimated) | Near zero for PRDs ≥ 95/100 | Meaningful |

### Quality lift

- **Consistency** — every PRD scored against the same 11 sections, the
  same weights, the same automation-readiness criteria
- **Auditability** — reviews posted to wiki as HTML comments;
  historical quality trend visible per team / per PM
- **Shared language** — "the PRD is 82/100, 3 blockers" is a clearer
  status than "the PRD needs work"

### Adoption metric

Recommended OKR if this tool is rolled out internally:

- **Primary:** % of PRDs scoring ≥ 95 at time of engineering kickoff,
  measured quarterly. Target: 80% by Q2 of adoption.
- **Secondary:** Average engineering kickoff meeting length. Target:
  < 45 minutes.
- **Guardrail:** PM satisfaction survey (quarterly) — "Does PRD
  Co-Pilot make your job easier?" Target: ≥ 70% agree.

---

## 9. Getting Started — for a product team

### One-time setup (per user)

```bash
# 1. Clone + install
git clone https://github.com/arimunandar/prd-reviewer-cli.git
cd prd-reviewer-cli
./install.sh          # prompts for Jira / Wiki / Figma credentials

# 2. Verify
prd-reviewer version  # → prd-reviewer 1.3.0
```

### Per-project setup

```bash
cd /path/to/your-project
prd-reviewer init     # installs .claude/skills/prd-reviewer + agent + CLAUDE.md section
```

### Daily use — three workflows

**Review an existing PRD:**
```
# In Claude Code
/prd-reviewer 76096147
```

**Generate a new PRD from a brief:**
```
/prd-reviewer write a PRD for a two-factor login feature
```

**Adjust an existing PRD with gaps:**
```
/prd-reviewer improve the PRD at https://wiki.example.com/pages/viewpage.action?pageId=12345
```

**Autonomous end-to-end (fetch → interview → review → wiki post):**
```
@prd-reviewer 76096147
```

### CLI cheat sheet

```bash
prd-reviewer prd fetch <id> --raw           # fetch PRD + download attachments
prd-reviewer prd rules                      # 11-section standard (markdown)
prd-reviewer prd rules --json               # same, JSON for programmatic use
prd-reviewer prd workflow                   # 7-step review workflow
prd-reviewer prd template                   # canonical blank PRD skeleton

prd-reviewer jira wiki page view <id>       # read any wiki page
prd-reviewer jira wiki page comment <id>    # post a comment
prd-reviewer figma url '<figma-url>'        # inspect a Figma node

prd-reviewer update                         # self-update from GitHub
```

---

## 10. Adoption Plan

### Phase 1 — Internal dogfooding (weeks 1–2)
- Install for 3–5 PMs on the product team
- Review the last 5 PRDs each PM shipped; record the score + time-saved
- Collect feedback; iterate on the 11-section weights if needed

### Phase 2 — Design partners (weeks 3–4)
- Expand to 10–15 PMs across 2 product areas
- Enforce the 95/100 threshold as a **soft gate** (eng can still
  start, but a PR link in the wiki comment requires the score)
- Track the adoption metric (see §8)

### Phase 3 — General availability (week 5+)
- Add to new-hire onboarding checklist
- Integrate into the weekly product sync (top 3 scores + top 3
  blockers as a standing agenda item)
- Consider making 95/100 a **hard gate** for sprint planning

### Risks during rollout

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| PMs see it as bureaucracy | Medium | High | Lead with time-saved, not compliance. Share the Engineer FAQ in every rollout demo |
| Tool feels too opinionated (weights, section names) | Low | Medium | Weights and section list are in-code and easy to tune; open to team input in Phase 1 |
| Wiki / Jira API changes | Low | High | Fetch is abstracted behind a single module; fixable in < 1 day |
| Claude API cost | Low | Low | Reviews are token-efficient (~5k input / 2k output per review); batch cost is negligible |

---

## 11. Roadmap

### v1.x — Near term (already shipped)
- ✅ 11-section compact standard
- ✅ CLI as data-provider architecture
- ✅ Three modes (Review / Generate / Adjust)
- ✅ AskUserQuestion interviews
- ✅ Engineer FAQ (6 categories, always on)
- ✅ Image / attachment auto-read
- ✅ Wiki HTML posting with full FAQ table

### v1.4 — Next (suggested)
- **Quality trend dashboard** — aggregate scores over time per team / per PM
- **Custom weights per team** — some teams weight Design Reference higher, some weight Acceptance Criteria higher
- **Multi-language PRD support** — currently tuned for English-first; expand to Bahasa / Mandarin
- **Slack integration** — post review summary to a Slack channel when a PRD clears 95/100

### v2.0 — Later
- **CI integration** — fail the build if a PRD linked in the commit message scores < 95 (configurable)
- **Cross-PRD consistency** — detect when two PRDs in the same quarter contradict each other on out-of-scope boundaries
- **PM coaching mode** — the AI suggests "next skill to learn" based on recurring review patterns per PM

---

## 12. FAQ (for your manager)

**Q: Who wrote this? Is it supported?**
A: Built by Ari Munandar. Open source (MIT), hosted at
github.com/arimunandar/prd-reviewer-cli. No vendor dependency.

**Q: What does it cost?**
A: The CLI is free. AI calls run through your existing Claude Code
subscription — no new line item.

**Q: Is our PRD data leaving our infra?**
A: PRDs are fetched from your Confluence, cached **locally** inside
the project repo (under `.prd-reviewer/`, which is `.gitignored`). The
AI review happens through Claude Code on the developer's machine. No
PRD content is stored by the tool itself.

**Q: What if our PRD format is different?**
A: The 11-section standard is opinionated by design — but the AI
reviews by **meaning**, not by section names. A PM who calls TL;DR
"Exec Summary" or Functional Requirements "Features" still gets
credited. For deeper customization, section weights live in
`src/commands/prd.rs` and take a 5-minute PR to change.

**Q: How long before we see results?**
A: Immediately on first review. The ROI is on the engineering side:
expect the first dropped-kickoff-clarification meeting within one
sprint of rollout.

**Q: What if we already have a PRD review culture?**
A: Great — this makes it faster and more consistent, not different.
Think of it as a calculator for the judgment you're already doing.

**Q: Can product management reviewers override the score?**
A: Yes. The score is an input to the conversation, not a verdict.
Section-level N/A notes let you explicitly mark a section as
non-applicable with zero deduction.

---

## 13. Appendix — Sample output

See `docs/SAMPLE_REVIEW.md` for a full review report on a real-world
PRD (page 76096147, "Lottie Animation For Unlocked Feature"), scored
24/100, with Engineer FAQ attached.

---

## 14. Contact & support

- **Repo:** https://github.com/arimunandar/prd-reviewer-cli
- **Issues:** GitHub Issues
- **Author:** Ari Munandar (arimunandar.dev@gmail.com)
- **License:** MIT
