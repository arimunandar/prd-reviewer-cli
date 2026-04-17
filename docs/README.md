# Documentation

Comprehensive reference for **PRD Co-Pilot** (`prd-reviewer`).

## Read in this order

1. **[PRODUCT_OVERVIEW.md](PRODUCT_OVERVIEW.md)** — the management
   pitch. Problem, solution, value, roadmap, adoption plan. Start here
   if you're sharing this with your manager.
2. **[SAMPLE_REVIEW.md](SAMPLE_REVIEW.md)** — a real, unedited review
   output on an internal PRD. Shows the exact format reviewers and
   engineers receive, including the Engineer FAQ.
3. **[ARCHITECTURE.md](ARCHITECTURE.md)** — technical reference for
   engineering reviewers: design decisions, code layout, dependencies,
   release flow, security.

## For users (skip to this)

- **[../README.md](../README.md)** — install + quick start
- CLI cheat sheet: `prd-reviewer --help` or see
  `PRODUCT_OVERVIEW.md` § 9

## For contributors

- Repo: https://github.com/arimunandar/prd-reviewer-cli
- Issues / discussions: GitHub
- Section weights + rules live in `src/commands/prd.rs::PRD_SECTIONS`
- Skill / agent text ships from `src/templates/mod.rs` via
  `prd-reviewer init` / `update`
