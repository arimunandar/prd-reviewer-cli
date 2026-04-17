use std::process::Command;

fn tuntun(args: &[&str]) -> (String, String, bool) {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .output()
        .expect("failed to run tuntun-ios");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. TOPIC REGISTRY — All topics exist and are findable
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn list_returns_all_topics() {
    let (out, _, ok) = tuntun(&["ls"]);
    assert!(ok);
    let topics = [
        "rules", "principles", "style", "architecture", "ui", "networking",
        "data", "domain", "di", "concurrency", "navigation", "testing",
        "security", "performance", "errorhandling", "accessibility", "swift",
        "xcodebuild", "refactor", "bugfix", "git", "migration", "common-bugs",
    ];
    for t in &topics {
        assert!(out.contains(t), "Missing topic: {}", t);
    }
}

#[test]
fn sections_command_lists_section_titles() {
    let (out, _, ok) = tuntun(&["sec", "architecture"]);
    assert!(ok);
    assert!(out.contains("VIP Overview"));
    assert!(out.contains("VIP Scene Structure"));
    assert!(out.contains("VIP Protocols"));
    assert!(out.contains("Factory"));
}

#[test]
fn sections_command_shows_numbered_index() {
    let (out, _, ok) = tuntun(&["sec", "rules"]);
    assert!(ok);
    assert!(out.contains("1."));
    assert!(out.contains("2."));
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. QUERY COMMAND — Topics, sections, brief mode, dot notation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn query_full_topic_includes_all_sections() {
    let (out, _, ok) = tuntun(&["q", "rules"]);
    assert!(ok);
    assert!(out.contains("# Code Quality Rules"));
    assert!(out.contains("Line Limits"));
    assert!(out.contains("VIP Strict Rules"));
}

#[test]
fn query_brief_mode_strips_code_blocks() {
    let (full, _, _) = tuntun(&["q", "architecture"]);
    let (brief, _, _) = tuntun(&["q", "architecture", "-b"]);
    // Brief should be significantly smaller
    assert!(
        brief.len() < full.len(),
        "Brief ({}) should be smaller than full ({})",
        brief.len(),
        full.len()
    );
    // Brief should still have section titles
    assert!(brief.contains("VIP Overview"));
}

#[test]
fn query_dot_notation_returns_single_section() {
    let (out, _, ok) = tuntun(&["q", "architecture.1"]);
    assert!(ok);
    assert!(out.contains("VIP Overview"));
    // Should NOT contain other sections
    assert!(!out.contains("## VIP Wiring"));
}

#[test]
fn query_dot_notation_with_brief() {
    let (out, _, ok) = tuntun(&["q", "rules.4", "-b"]);
    assert!(ok);
    assert!(out.contains("VIP Strict Rules"));
}

#[test]
fn query_section_filter_by_name() {
    let (out, _, ok) = tuntun(&["q", "style", "-s", "Naming"]);
    assert!(ok);
    assert!(out.contains("Naming Conventions"));
    // Should NOT contain unrelated sections
    assert!(!out.contains("## Formatting Rules"));
}

#[test]
fn query_multiple_topics() {
    let (out, _, ok) = tuntun(&["q", "rules,style"]);
    assert!(ok);
    assert!(out.contains("Code Quality Rules"));
    assert!(out.contains("Naming Conventions"));
}

#[test]
fn query_case_insensitive() {
    let (out, _, ok) = tuntun(&["q", "ARCHITECTURE"]);
    assert!(ok);
    assert!(out.contains("VIP Overview"));
}

#[test]
fn query_invalid_topic_shows_available() {
    let (out, stderr, _) = tuntun(&["q", "nonexistent"]);
    let combined = format!("{}{}", out, stderr);
    // Should show "Unknown topic" and list available topics
    assert!(
        combined.contains("Unknown topic") || combined.contains("rules"),
        "Should show error with available topics, got: {}", combined
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. SEARCH — Keyword matching, relevance ranking
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn search_vip_returns_architecture_first() {
    let (out, _, ok) = tuntun(&["s", "VIP"]);
    assert!(ok);
    // Architecture should appear (VIP is core to it)
    assert!(out.contains("architecture"));
}

#[test]
fn search_factory_finds_architecture_and_di() {
    let (out, _, ok) = tuntun(&["s", "factory"]);
    assert!(ok);
    assert!(out.contains("architecture") || out.contains("di"));
}

#[test]
fn search_interactor_finds_domain() {
    let (out, _, ok) = tuntun(&["s", "interactor"]);
    assert!(ok);
    assert!(out.contains("domain"));
}

#[test]
fn search_snapkit_finds_ui() {
    let (out, _, ok) = tuntun(&["s", "SnapKit"]);
    assert!(ok);
    assert!(out.contains("ui"));
}

#[test]
fn search_no_results() {
    let (out, _, _) = tuntun(&["s", "zzzznotfound"]);
    // CLI exits non-zero on no results, output contains "No topics found"
    assert!(
        out.contains("No topics found") || out.contains("no matches") || out.is_empty(),
        "Should indicate no results, got: {}", out
    );
}

#[test]
fn search_presenter_finds_domain_and_architecture() {
    let (out, _, ok) = tuntun(&["s", "presenter"]);
    assert!(ok);
    assert!(out.contains("domain"));
}

#[test]
fn search_sendable_finds_relevant() {
    let (out, _, ok) = tuntun(&["s", "Sendable"]);
    assert!(ok);
    // Sendable is mentioned in rules, domain, architecture
    assert!(
        out.contains("rules") || out.contains("domain") || out.contains("architecture"),
        "Sendable search should find rules/domain/architecture"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. FOR TASK — Task profiles, correct sections loaded
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn for_list_shows_all_tasks() {
    let (out, _, ok) = tuntun(&["for", "--list"]);
    assert!(ok);
    let tasks = [
        "viewcontroller", "interactor", "presenter", "worker", "cell",
        "scene", "router", "models", "test", "bugfix", "refactor",
        "api", "view", "pagination", "form", "async", "di", "error",
        "security", "performance",
    ];
    for t in &tasks {
        assert!(out.contains(t), "Missing task: {}", t);
    }
}

#[test]
fn for_viewcontroller_loads_correct_rules() {
    let (out, _, ok) = tuntun(&["for", "viewcontroller"]);
    assert!(ok);
    // Should include Line Limits (rules.1)
    assert!(out.contains("Line Limits") || out.contains("30 lines"));
    // Should include ViewController structure (ui.1)
    assert!(out.contains("ViewController") || out.contains("viewDidLoad"));
}

#[test]
fn for_interactor_includes_vip_rules() {
    let (out, _, ok) = tuntun(&["for", "interactor"]);
    assert!(ok);
    assert!(out.contains("VIP") || out.contains("Interactor"));
    assert!(out.contains("@MainActor") || out.contains("MainActor"));
}

#[test]
fn for_presenter_includes_formatting_rules() {
    let (out, _, ok) = tuntun(&["for", "presenter"]);
    assert!(ok);
    assert!(out.contains("Presenter") || out.contains("format"));
}

#[test]
fn for_cell_includes_performance_rules() {
    let (out, _, ok) = tuntun(&["for", "cell"]);
    assert!(ok);
    // Cell task should mention cellForRowAt or pre-compute
    assert!(
        out.contains("cellForRowAt") || out.contains("pre-compute") || out.contains("Component"),
        "Cell task should include performance/component rules"
    );
}

#[test]
fn for_scene_includes_all_vip_components() {
    let (out, _, ok) = tuntun(&["for", "scene"]);
    assert!(ok);
    // Scene should reference VIP structure
    assert!(out.contains("Factory") || out.contains("Scene Structure") || out.contains("VIP"));
}

#[test]
fn for_security_includes_security_rules() {
    let (out, _, ok) = tuntun(&["for", "security"]);
    assert!(ok);
    assert!(
        out.contains("Keychain") || out.contains("token") || out.contains("security") || out.contains("Security"),
        "Security task should include keychain/token rules"
    );
}

#[test]
fn for_bugfix_includes_workflow() {
    let (out, _, ok) = tuntun(&["for", "bugfix"]);
    assert!(ok);
    assert!(
        out.contains("REPRODUCE") || out.contains("reproduce") || out.contains("Bug Fix"),
        "Bugfix task should include workflow steps"
    );
}

#[test]
fn for_full_flag_includes_code_examples() {
    let (brief, _, _) = tuntun(&["for", "viewcontroller"]);
    let (full, _, _) = tuntun(&["for", "viewcontroller", "--full"]);
    assert!(
        full.len() > brief.len(),
        "Full ({}) should be larger than brief ({})",
        full.len(),
        brief.len()
    );
}

#[test]
fn for_unknown_task_shows_available() {
    let (out, stderr, _) = tuntun(&["for", "zznotfound"]);
    let combined = format!("{}{}", out, stderr);
    // Should show available tasks when no match
    assert!(combined.contains("viewcontroller"), "Should list viewcontroller as suggestion");
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. REVIEW COMMAND — Checklists, focus areas
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn review_brief_outputs_checklists() {
    let (out, _, ok) = tuntun(&["review", "-b"]);
    assert!(ok);
    assert!(out.contains("Line Limits") || out.contains("30 lines"));
}

#[test]
fn review_focus_security() {
    let (out, _, ok) = tuntun(&["review", "-b", "-f", "security"]);
    assert!(ok);
    assert!(
        out.contains("Security") || out.contains("security"),
        "Security focus should include security checklist"
    );
}

#[test]
fn review_focus_performance() {
    let (out, _, ok) = tuntun(&["review", "-b", "-f", "performance"]);
    assert!(ok);
    assert!(
        out.contains("Performance") || out.contains("performance"),
        "Performance focus should include performance checklist"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. CONTENT RULES — VIP rules are present and correct
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn vip_strict_rules_no_swiftui() {
    let (out, _, ok) = tuntun(&["q", "rules.4"]);
    assert!(ok);
    assert!(
        out.contains("SwiftUI") || out.contains("UIKit"),
        "VIP strict rules should mention SwiftUI prohibition or UIKit requirement"
    );
}

#[test]
fn vip_strict_rules_no_combine() {
    // Combine prohibition is in style topic (Swift Idioms section)
    let (out, _, ok) = tuntun(&["q", "style", "-s", "Idioms"]);
    assert!(ok);
    assert!(
        out.contains("Combine") || out.contains("combine") || out.contains("NO Combine"),
        "Swift Idioms should mention Combine prohibition, got: {}",
        &out[..out.len().min(500)]
    );
}

#[test]
fn vip_strict_rules_mainactor() {
    let (out, _, ok) = tuntun(&["q", "rules.4"]);
    assert!(ok);
    assert!(
        out.contains("@MainActor") || out.contains("MainActor"),
        "VIP strict rules should mention @MainActor"
    );
}

#[test]
fn vip_strict_rules_final_classes() {
    let (out, _, ok) = tuntun(&["q", "rules.4"]);
    assert!(ok);
    assert!(
        out.contains("final"),
        "VIP strict rules should mention final classes"
    );
}

#[test]
fn vip_strict_rules_injected() {
    let (out, _, ok) = tuntun(&["q", "rules.4"]);
    assert!(ok);
    assert!(
        out.contains("@Injected") || out.contains("Injected"),
        "VIP strict rules should mention @Injected"
    );
}

#[test]
fn line_limits_30_300_400() {
    let (out, _, ok) = tuntun(&["q", "rules.1"]);
    assert!(ok);
    assert!(out.contains("30"), "Should mention 30 lines per function");
    assert!(out.contains("300"), "Should mention 300 lines per class");
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. ARCHITECTURE CONTENT — VIP patterns correct
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn architecture_vip_data_flow() {
    let (out, _, ok) = tuntun(&["q", "architecture.1"]);
    assert!(ok);
    // Should describe the VIP cycle
    assert!(out.contains("ViewController") || out.contains("View"));
    assert!(out.contains("Interactor"));
    assert!(out.contains("Presenter"));
}

#[test]
fn architecture_scene_structure_files() {
    let (out, _, ok) = tuntun(&["q", "architecture.2"]);
    assert!(ok);
    // Scene should have these files
    assert!(out.contains("Models"));
    assert!(out.contains("Factory"));
    assert!(out.contains("Router") || out.contains("Routing"));
}

#[test]
fn architecture_factory_pattern() {
    let (out, _, ok) = tuntun(&["q", "architecture.4"]);
    assert!(ok);
    assert!(out.contains("Factory"));
    assert!(out.contains("@Injected") || out.contains("Injected"));
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. STYLE CONTENT — Naming conventions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn style_naming_vip_suffixes() {
    let (out, _, ok) = tuntun(&["q", "style.1"]);
    assert!(ok);
    assert!(out.contains("DisplayLogic") || out.contains("Protocol"));
}

#[test]
fn style_no_combine_no_swiftui() {
    let (out, _, ok) = tuntun(&["q", "style"]);
    assert!(ok);
    // Style should enforce no Combine/SwiftUI
    assert!(
        out.contains("Combine") || out.contains("SwiftUI") || out.contains("async/await"),
        "Style should mention Combine/SwiftUI prohibition or async/await preference"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. DOMAIN CONTENT — Interactor, Presenter, Worker patterns
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn domain_interactor_no_uikit() {
    let (out, _, ok) = tuntun(&["q", "domain.1"]);
    assert!(ok);
    assert!(
        out.contains("UIKit") || out.contains("import"),
        "Interactor rules should mention no UIKit import"
    );
}

#[test]
fn domain_presenter_weak_reference() {
    let (out, _, ok) = tuntun(&["q", "domain.2"]);
    assert!(ok);
    assert!(
        out.contains("weak") || out.contains("Weak"),
        "Presenter rules should mention weak ViewController reference"
    );
}

#[test]
fn domain_presenter_precompute() {
    let (out, _, ok) = tuntun(&["q", "domain.6"]);
    assert!(ok);
    assert!(
        out.contains("cellForRowAt") || out.contains("pre-comput") || out.contains("Pre-Comput"),
        "Presenter should mention pre-computing display values"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. FIGMA CLI — URL parsing, config
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn figma_help_shows_all_subcommands() {
    let (out, _, ok) = tuntun(&["figma", "--help"]);
    assert!(ok);
    let cmds = ["url", "view", "file", "node", "comment", "team", "project", "variable", "screenshot"];
    for cmd in &cmds {
        assert!(out.contains(cmd), "Missing figma subcommand: {}", cmd);
    }
}

#[test]
fn figma_url_no_node_returns_parsed_metadata() {
    let (out, _, ok) = tuntun(&["figma", "url", "https://www.figma.com/design/abc123DEF/My-Design"]);
    assert!(ok);
    assert!(out.contains("abc123DEF"));
    assert!(out.contains("file_key"));
}

#[test]
fn figma_url_with_node_parses_node_id() {
    // This will fail with API error (fake key) but the URL parsing should work
    let (out, stderr, _) = tuntun(&["figma", "url", "https://www.figma.com/design/abc123DEF/My-Design?node-id=1-23"]);
    // Should either succeed or fail with API error (not parse error)
    let combined = format!("{}{}", out, stderr);
    assert!(
        combined.contains("API error") || combined.contains("1:23") || combined.contains("abc123DEF"),
        "Should parse URL correctly, got: {}", combined
    );
}

#[test]
fn figma_url_plain_file_key() {
    let (out, _, ok) = tuntun(&["figma", "url", "abc123DEF"]);
    assert!(ok);
    assert!(out.contains("abc123DEF"));
    assert!(out.contains("file_key"));
}

#[test]
fn figma_url_branch_url() {
    let (out, _, ok) = tuntun(&["figma", "url", "https://www.figma.com/design/PARENT/branch/BRANCH123/Name"]);
    assert!(ok);
    assert!(out.contains("BRANCH123"));
    assert!(out.contains("is_branch"));
}

#[test]
fn figma_config_show_works() {
    let (out, stderr, _) = tuntun(&["figma", "config", "show"]);
    let combined = format!("{}{}", out, stderr);
    // Should either show config or say config not found
    assert!(
        combined.contains("Personal Token") || combined.contains("config"),
        "Config show should display token info or config error"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. INIT COMMAND — Generates correct files
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn init_creates_expected_files() {
    let dir = std::env::temp_dir().join("tuntun_test_init");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let (out, _, ok) = tuntun(&["init", "-p", dir.to_str().unwrap()]);
    assert!(ok, "init should succeed");

    // Check files exist
    assert!(dir.join("CLAUDE.md").exists(), "CLAUDE.md should exist");
    assert!(dir.join(".claude/agents/ios-conventions.md").exists());
    assert!(dir.join(".claude/skills/ios/SKILL.md").exists());
    assert!(dir.join(".claude/skills/ios-architecture/SKILL.md").exists());
    assert!(dir.join(".claude/skills/ios-review/SKILL.md").exists());
    assert!(dir.join(".claude/skills/ios-figma/SKILL.md").exists());
    assert!(dir.join(".claude/skills/ios-security/SKILL.md").exists());
    assert!(dir.join(".claude/skills/ios-performance/SKILL.md").exists());
    assert!(dir.join(".claude/skills/ios-bugfix/SKILL.md").exists());
    assert!(dir.join(".claude/skills/ios-refactor/SKILL.md").exists());
    assert!(dir.join(".claude/skills/ios-migrate/SKILL.md").exists());
    assert!(dir.join(".claude/skills/ios-vip-migrate/SKILL.md").exists());
    assert!(dir.join(".claude/skills/ios-xcodebuild/SKILL.md").exists());

    // Check CLAUDE.md content
    let claude_md = std::fs::read_to_string(dir.join("CLAUDE.md")).unwrap();
    assert!(claude_md.contains("tuntun-ios"), "CLAUDE.md should reference tuntun-ios");
    assert!(claude_md.contains("VIP"), "CLAUDE.md should mention VIP");
    assert!(claude_md.contains("figma") || claude_md.contains("Figma"), "CLAUDE.md should mention Figma");

    // Check skill content has correct allowed-tools
    let ios_skill = std::fs::read_to_string(dir.join(".claude/skills/ios/SKILL.md")).unwrap();
    assert!(ios_skill.contains("Bash(tuntun-ios *)"), "ios skill should auto-approve tuntun-ios");
    assert!(ios_skill.contains("Write"), "ios skill should allow Write");

    let figma_skill = std::fs::read_to_string(dir.join(".claude/skills/ios-figma/SKILL.md")).unwrap();
    assert!(figma_skill.contains("Bash(tuntun-ios *)"), "figma skill should auto-approve tuntun-ios");

    let review_skill = std::fs::read_to_string(dir.join(".claude/skills/ios-review/SKILL.md")).unwrap();
    assert!(!review_skill.contains("Write"), "review skill should NOT allow Write");

    // Check agent is read-only
    let agent = std::fs::read_to_string(dir.join(".claude/agents/ios-conventions.md")).unwrap();
    assert!(!agent.contains("Write"), "agent should NOT allow Write");
    assert!(agent.contains("TRIGGER ONLY"), "agent should have strict trigger");

    // Output should mention all created files
    assert!(out.contains("[created] CLAUDE.md"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn init_updates_existing_files() {
    let dir = std::env::temp_dir().join("tuntun_test_init_update");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Run init twice
    tuntun(&["init", "-p", dir.to_str().unwrap()]);
    let (out, _, ok) = tuntun(&["init", "-p", dir.to_str().unwrap()]);
    assert!(ok);
    // Second run should update skills (not skip)
    assert!(out.contains("[updated]"), "Re-init should update existing skills");
    // Skills should NOT be skipped anymore
    assert!(!out.contains("[skip] .claude/skills/"), "Skills should be updated, not skipped");

    let _ = std::fs::remove_dir_all(&dir);
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. SKILL SEPARATION — No overlapping triggers
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn skills_have_non_overlapping_triggers() {
    let dir = std::env::temp_dir().join("tuntun_test_triggers");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    tuntun(&["init", "-p", dir.to_str().unwrap()]);

    // ios skill should NOT contain "code review" trigger
    let ios = std::fs::read_to_string(dir.join(".claude/skills/ios/SKILL.md")).unwrap();
    assert!(ios.contains("Do NOT trigger for"), "ios should have exclusion list");

    // ios-review should NOT contain "write" or "modify" trigger
    let review = std::fs::read_to_string(dir.join(".claude/skills/ios-review/SKILL.md")).unwrap();
    assert!(!review.contains("Write, Edit"), "review should NOT have Write/Edit");

    // ios-figma should NOT trigger on generic "design" without Figma context
    let figma = std::fs::read_to_string(dir.join(".claude/skills/ios-figma/SKILL.md")).unwrap();
    assert!(figma.contains("Do NOT trigger for generic"), "figma should exclude generic design requests");

    // agent should say TRIGGER ONLY
    let agent = std::fs::read_to_string(dir.join(".claude/agents/ios-conventions.md")).unwrap();
    assert!(agent.contains("TRIGGER ONLY"), "agent should have strict trigger");

    let _ = std::fs::remove_dir_all(&dir);
}

// ─────────────────────────────────────────────────────────────────────────────
// 13. VERSION
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn version_shows_current_version() {
    let (out, _, ok) = tuntun(&["version"]);
    assert!(ok);
    assert!(out.contains("tuntun-ios"));
    assert!(out.contains("0.3.0") || out.contains("0."));
}

// ─────────────────────────────────────────────────────────────────────────────
// 14. JIRA CLI — Subcommands, config
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn jira_help_shows_all_subcommands() {
    let (out, _, ok) = tuntun(&["jira", "--help"]);
    assert!(ok);
    assert!(out.contains("issue"));
    assert!(out.contains("wiki"));
}

#[test]
fn jira_issue_help_shows_operations() {
    let (out, _, ok) = tuntun(&["jira", "issue", "issue", "--help"]);
    assert!(ok);
    let cmds = ["view", "search", "create", "update", "comment", "transition"];
    for cmd in &cmds {
        assert!(out.contains(cmd), "Missing jira issue subcommand: {}", cmd);
    }
}

#[test]
fn jira_wiki_help_shows_operations() {
    let (out, _, ok) = tuntun(&["jira", "wiki", "--help"]);
    assert!(ok);
    assert!(out.contains("page"));
    assert!(out.contains("space"));
}

#[test]
fn jira_wiki_page_help_shows_operations() {
    let (out, _, ok) = tuntun(&["jira", "wiki", "page", "--help"]);
    assert!(ok);
    let cmds = ["view", "search", "create", "update", "export"];
    for cmd in &cmds {
        assert!(out.contains(cmd), "Missing wiki page subcommand: {}", cmd);
    }
}

#[test]
fn jira_config_show_works() {
    let (out, stderr, _) = tuntun(&["jira", "config", "show"]);
    let combined = format!("{}{}", out, stderr);
    assert!(
        combined.contains("Jira Token") || combined.contains("Jira Base URL") || combined.contains("config"),
        "Config show should display token info or config error"
    );
}

#[test]
fn jira_issue_agile_help() {
    let (out, _, ok) = tuntun(&["jira", "issue", "--help"]);
    assert!(ok);
    assert!(out.contains("board"), "Should have board subcommand");
    assert!(out.contains("sprint"), "Should have sprint subcommand");
    assert!(out.contains("project"), "Should have project subcommand");
}
