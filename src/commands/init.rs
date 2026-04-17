use crate::config::TuntunConfig;
use crate::templates;
use std::fs;
use std::path::Path;

const TUNTUN_MARKER: &str = "<!-- prd-reviewer -->";

pub fn run(path: &str) {
    let target = Path::new(path);

    if !target.exists() {
        eprintln!("Directory does not exist: {}", path);
        std::process::exit(1);
    }

    sync_files(target, false);

    check_config();

    println!();
    println!("prd-reviewer initialized in: {}", target.display());
    println!();
    println!("CLI Quick Start:");
    println!("  prd-reviewer prd fetch <PAGE_ID> --raw   — fetch raw PRD");
    println!("  prd-reviewer prd rules                   — 11-section rules (markdown)");
    println!("  prd-reviewer prd rules --json            — rules as JSON (for AI)");
    println!("  prd-reviewer prd workflow                — step-by-step review workflow");
    println!("  prd-reviewer prd template                — PRD template v2");
    println!("  prd-reviewer jira wiki page view <ID>    — view wiki page");
    println!("  prd-reviewer figma url '<URL>'           — inspect Figma design");
    println!();
    println!("Skill (1) & Agent (1):");
    println!("  /prd-reviewer                — deep PRD quality review");
    println!("  @prd-reviewer                — autonomous end-to-end PRD audit");
}

/// Sync all generated files to the target directory.
/// Called by both `init` and `update` (via sync_project).
pub fn sync_files(target: &Path, quiet: bool) {
    let claude_dir = target.join(".claude");
    let agents_dir = claude_dir.join("agents");
    let skills_dir = claude_dir.join("skills").join("prd-reviewer");
    fs::create_dir_all(&agents_dir).ok();
    fs::create_dir_all(&skills_dir).ok();

    // .tuntun dir for cached wiki PRDs
    let tuntun_dir = target.join(".tuntun");
    fs::create_dir_all(tuntun_dir.join("prd")).ok();

    // CLAUDE.md: create or replace prd-reviewer section (preserves user content)
    let claude_path = target.join("CLAUDE.md");
    write_or_replace_claude_md(&claude_path, templates::claude_md(), quiet);

    // Agent — always overwrite (generated content)
    let agent_file = agents_dir.join("prd-reviewer.md");
    write_always(&agent_file, templates::agent_md(), ".claude/agents/prd-reviewer.md", quiet);

    // Skill — always overwrite
    let skill_file = skills_dir.join("SKILL.md");
    write_always(&skill_file, templates::skill_prd_reviewer(), ".claude/skills/prd-reviewer/SKILL.md", quiet);
}

/// Check if a directory has prd-reviewer files (used by update to find projects).
pub fn has_tuntun_files(path: &Path) -> bool {
    path.join(".claude/skills/prd-reviewer/SKILL.md").exists()
        || path.join("CLAUDE.md")
            .exists()
            .then(|| {
                fs::read_to_string(path.join("CLAUDE.md"))
                    .map(|c| c.contains(TUNTUN_MARKER))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
}

fn check_config() {
    let config_path = TuntunConfig::default_path();
    match TuntunConfig::load() {
        Ok(cfg) => {
            let has_jira = !cfg.jira.access_token.is_empty();
            let has_wiki = !cfg.wiki.access_token.is_empty();
            let has_figma = !cfg.figma.personal_token.is_empty();
            let mut parts = Vec::new();
            if has_jira { parts.push("Jira"); }
            if has_wiki { parts.push("Wiki"); }
            if has_figma { parts.push("Figma"); }
            let status = if parts.is_empty() {
                "No credentials configured".to_string()
            } else {
                format!("{} configured", parts.join(" + "))
            };
            println!("  [config] {} ({})", config_path.display(), status);
            if !has_jira || !has_wiki || !has_figma {
                println!("  Run install.sh to set up missing credentials");
            }
        }
        Err(_) => {
            println!("  [warning] No config found at {}", config_path.display());
            println!("  Run install.sh to set up credentials");
        }
    }
}

fn write_or_replace_claude_md(path: &Path, content: &str, quiet: bool) {
    let marker_start = format!("\n{}\n", TUNTUN_MARKER);
    let new_section = format!("{}{}", marker_start, content);

    if path.exists() {
        let existing = fs::read_to_string(path).unwrap_or_default();

        if let Some(marker_pos) = existing.find(TUNTUN_MARKER) {
            let user_content = existing[..marker_pos].trim_end();
            let combined = format!("{}\n{}", user_content, new_section);
            fs::write(path, combined).unwrap_or_else(|e| {
                eprintln!("Failed to update CLAUDE.md: {}", e);
                std::process::exit(1);
            });
            if !quiet {
                println!("  [updated] CLAUDE.md — replaced prd-reviewer section");
            }
        } else {
            let combined = format!("{}\n{}", existing.trim_end(), new_section);
            fs::write(path, combined).unwrap_or_else(|e| {
                eprintln!("Failed to append to CLAUDE.md: {}", e);
                std::process::exit(1);
            });
            if !quiet {
                println!("  [appended] CLAUDE.md — added prd-reviewer section");
            }
        }
    } else {
        let new_content = format!("{}\n{}", TUNTUN_MARKER, content);
        fs::write(path, new_content).unwrap_or_else(|e| {
            eprintln!("Failed to create CLAUDE.md: {}", e);
            std::process::exit(1);
        });
        if !quiet {
            println!("  [created] CLAUDE.md");
        }
    }
}

fn write_always(path: &Path, content: &str, label: &str, quiet: bool) {
    let existed = path.exists();
    fs::write(path, content).unwrap_or_else(|e| {
        eprintln!("Failed to write {}: {}", label, e);
        std::process::exit(1);
    });
    if !quiet {
        if existed {
            println!("  [updated] {}", label);
        } else {
            println!("  [created] {}", label);
        }
    }
}
