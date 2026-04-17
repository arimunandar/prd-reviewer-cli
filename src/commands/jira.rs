use clap::Subcommand;

use crate::figma::output::{self, OutputFormat};
use crate::jira::api::{jira_api, wiki_api};
use crate::jira::client::Client;
use crate::jira::config::Config;
use crate::jira::confluence;
use crate::jira::error::JiraError;
use crate::jira::models::jira::*;
use crate::jira::models::wiki::*;

#[derive(Subcommand)]
pub enum JiraCommands {
    /// Jira issue, project, board, sprint commands
    #[command(alias = "jira")]
    Issue {
        #[command(subcommand)]
        command: JiraIssueParent,
    },
    /// Confluence Wiki page and space commands
    Wiki {
        #[command(subcommand)]
        command: WikiParent,
    },
}

// ─── Jira ───────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum JiraIssueParent {
    /// Issue operations
    Issue {
        #[command(subcommand)]
        command: IssueCommands,
    },
    /// List Jira projects
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    /// Board operations (Agile)
    Board {
        #[command(subcommand)]
        command: BoardCommands,
    },
    /// Sprint operations (Agile)
    Sprint {
        #[command(subcommand)]
        command: SprintCommands,
    },
}

#[derive(Subcommand)]
pub enum IssueCommands {
    /// View issue details
    View {
        /// Issue key (e.g. PROJ-123)
        key: String,
        /// Include comments
        #[arg(long)]
        comments: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Skip TLS verification
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Search issues with JQL
    Search {
        /// JQL query
        jql: String,
        /// Maximum results
        #[arg(long, default_value = "50")]
        max_results: i32,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Skip TLS verification
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Create a new issue
    Create {
        /// Project key
        #[arg(long)]
        project: String,
        /// Issue type
        #[arg(long, default_value = "Task")]
        r#type: String,
        /// Issue summary
        #[arg(long)]
        summary: String,
        /// Issue description
        #[arg(long, default_value = "")]
        description: String,
        /// Skip TLS verification
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Update issue fields
    Update {
        /// Issue key
        key: String,
        /// New summary
        #[arg(long)]
        summary: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New assignee username
        #[arg(long)]
        assignee: Option<String>,
        /// Skip TLS verification
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Add a comment to an issue
    Comment {
        /// Issue key
        key: String,
        /// Comment body
        #[arg(long, default_value = "")]
        body: String,
        /// Read comment body from file
        #[arg(long, default_value = "")]
        file: String,
        /// Skip TLS verification
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Change issue status
    Transition {
        /// Issue key
        key: String,
        /// Target status name
        #[arg(long)]
        status: Option<String>,
        /// List available transitions
        #[arg(long)]
        list: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Skip TLS verification
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// List all projects
    List {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
}

#[derive(Subcommand)]
pub enum BoardCommands {
    /// List boards
    List {
        /// Filter by project key
        #[arg(long, default_value = "")]
        project: String,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
}

#[derive(Subcommand)]
pub enum SprintCommands {
    /// List sprints for a board
    List {
        /// Board ID
        #[arg(long)]
        board: i32,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
}

// ─── Wiki ───────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum WikiParent {
    /// Page operations
    Page {
        #[command(subcommand)]
        command: PageCommands,
    },
    /// Space operations
    Space {
        #[command(subcommand)]
        command: SpaceCommands,
    },
}

#[derive(Subcommand)]
pub enum PageCommands {
    /// View page content
    View {
        /// Page ID
        id: String,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Search pages
    Search {
        /// Search by title
        #[arg(long, default_value = "")]
        title: String,
        /// CQL query
        #[arg(long, default_value = "")]
        cql: String,
        /// Filter by space key
        #[arg(long, default_value = "")]
        space: String,
        /// Maximum results
        #[arg(long, default_value = "50")]
        max_results: i32,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Create a page
    Create {
        /// Space key
        #[arg(long)]
        space: String,
        /// Page title
        #[arg(long)]
        title: String,
        /// Page body (HTML)
        #[arg(long, default_value = "")]
        body: String,
        /// Parent page ID
        #[arg(long, default_value = "")]
        parent: String,
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Update a page
    Update {
        /// Page ID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New body (HTML)
        #[arg(long)]
        body: Option<String>,
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Add a comment to a wiki page
    Comment {
        /// Page ID
        id: String,
        /// Comment body (HTML). Use --file to read from file instead
        #[arg(long, default_value = "")]
        body: String,
        /// Read comment body from file (HTML)
        #[arg(long, default_value = "")]
        file: String,
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
    /// Export page to PDF or HTML
    Export {
        /// Page ID
        id: String,
        /// Export format (pdf|html)
        #[arg(long, default_value = "pdf")]
        format: String,
        /// Output file path
        #[arg(long, default_value = "")]
        output: String,
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
}

#[derive(Subcommand)]
pub enum SpaceCommands {
    /// List wiki spaces
    List {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "true")]
        insecure: bool,
    },
}

// ─── Dispatch ───────────────────────────────────────────────────────────────

pub fn run(cmd: JiraCommands) {
    if let Err(e) = run_inner(cmd) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_inner(cmd: JiraCommands) -> Result<(), JiraError> {
    match cmd {
        JiraCommands::Issue { command } => run_jira(command),
        JiraCommands::Wiki { command } => run_wiki(command),
    }
}

// ─── Jira Commands ──────────────────────────────────────────────────────────

fn run_jira(cmd: JiraIssueParent) -> Result<(), JiraError> {
    match cmd {
        JiraIssueParent::Issue { command } => run_issue(command),
        JiraIssueParent::Project { command } => run_project(command),
        JiraIssueParent::Board { command } => run_board(command),
        JiraIssueParent::Sprint { command } => run_sprint(command),
    }
}

fn run_issue(cmd: IssueCommands) -> Result<(), JiraError> {
    match cmd {
        IssueCommands::View { key, comments, json, insecure } => {
            let client = new_client(insecure)?;
            let expand = if comments { "renderedFields" } else { "" };
            let mut issue = jira_api::get_issue(&client, &key, expand)?;

            // Build attachment map
            let mut attach_map = confluence::AttachmentMap::new();
            for att in &issue.fields.attachment {
                attach_map.insert(att.filename.clone(), att.content.clone());
            }

            // Convert description
            if !issue.fields.description.is_empty() {
                let md = confluence::convert_jira_wiki(&issue.fields.description, &attach_map);
                let md = confluence::download_images(&md, &issue.key, &client);
                issue.fields.description = md;
            }

            // Convert comments
            if let Some(ref mut comment_list) = issue.fields.comment {
                for cm in &mut comment_list.comments {
                    let md = confluence::convert_jira_wiki(&cm.body, &attach_map);
                    let md = confluence::download_images(&md, &issue.key, &client);
                    cm.body = md;
                }
            }

            if json {
                output::render_json(&issue);
                return Ok(());
            }

            // Build full markdown
            let mut full_md = String::new();
            full_md.push_str(&format!("# {}\n\n", issue.key));
            full_md.push_str(&format!("**{}**\n\n", issue.fields.summary));
            full_md.push_str("| Field | Value |\n| --- | --- |\n");
            if let Some(ref t) = issue.fields.issuetype {
                full_md.push_str(&format!("| Type | {} |\n", t.name));
            }
            if let Some(ref s) = issue.fields.status {
                full_md.push_str(&format!("| Status | {} |\n", s.name));
            }
            if let Some(ref p) = issue.fields.priority {
                full_md.push_str(&format!("| Priority | {} |\n", p.name));
            }
            if let Some(ref a) = issue.fields.assignee {
                full_md.push_str(&format!("| Assignee | {} |\n", a.display_name));
            }
            if let Some(ref r) = issue.fields.reporter {
                full_md.push_str(&format!("| Reporter | {} |\n", r.display_name));
            }
            full_md.push_str(&format!("| Created | {} |\n", issue.fields.created));
            full_md.push_str(&format!("| Updated | {} |\n", issue.fields.updated));

            if !issue.fields.description.is_empty() {
                full_md.push_str("\n---\n\n");
                full_md.push_str(&issue.fields.description);
                full_md.push('\n');
            }

            if comments {
                if let Some(ref cl) = issue.fields.comment {
                    full_md.push_str(&format!("\n---\n\n## Comments ({})\n\n", cl.comments.len()));
                    for cm in &cl.comments {
                        let author = cm.author.as_ref().map(|a| a.display_name.as_str()).unwrap_or("Unknown");
                        full_md.push_str(&format!("**{}** - _{}_\n\n", author, cm.created));
                        full_md.push_str(&format!("{}\n\n", cm.body));
                    }
                }
            }

            print!("{}", full_md);

            // Auto-save to .prd-reviewer/tasks/<key>.md
            save_task(&issue.key, &full_md);

            Ok(())
        }

        IssueCommands::Search { jql, max_results, json, insecure } => {
            let client = new_client(insecure)?;
            let result = jira_api::search_issues(&client, &jql, max_results)?;
            if !json {
                println!("Total: {}", result.total);
            }
            let fmt = if json { OutputFormat::Json } else { OutputFormat::Table };
            output::render(&result.issues, fmt);
            Ok(())
        }

        IssueCommands::Create { project, r#type, summary, description, insecure } => {
            let client = new_client(insecure)?;
            let mut fields = std::collections::HashMap::new();
            fields.insert("project".to_string(), serde_json::json!({"key": project}));
            fields.insert("issuetype".to_string(), serde_json::json!({"name": r#type}));
            fields.insert("summary".to_string(), serde_json::json!(summary));
            if !description.is_empty() {
                fields.insert("description".to_string(), serde_json::json!(description));
            }
            let payload = JiraCreateIssue { fields };
            let issue = jira_api::create_issue(&client, &payload)?;
            println!("Created issue: {}", issue.key);
            Ok(())
        }

        IssueCommands::Update { key, summary, description, assignee, insecure } => {
            let client = new_client(insecure)?;
            let mut fields = std::collections::HashMap::new();
            if let Some(s) = summary { fields.insert("summary".to_string(), serde_json::json!(s)); }
            if let Some(d) = description { fields.insert("description".to_string(), serde_json::json!(d)); }
            if let Some(a) = assignee { fields.insert("assignee".to_string(), serde_json::json!({"name": a})); }
            if fields.is_empty() {
                return Err(JiraError::Config("no fields to update; use --summary, --description, or --assignee".to_string()));
            }
            let payload = JiraCreateIssue { fields };
            jira_api::update_issue(&client, &key, &payload)?;
            println!("Updated issue: {}", key);
            Ok(())
        }

        IssueCommands::Comment { key, body, file, insecure } => {
            let client = new_client(insecure)?;
            let text = if !file.is_empty() {
                std::fs::read_to_string(&file)?
            } else if !body.is_empty() {
                body
            } else {
                return Err(JiraError::Config("Provide --body or --file".to_string()));
            };
            jira_api::add_comment(&client, &key, &text)?;
            println!("Comment added to {}", key);
            Ok(())
        }

        IssueCommands::Transition { key, status, list, json, insecure } => {
            let client = new_client(insecure)?;
            let transitions = jira_api::get_transitions(&client, &key)?;

            if list {
                let fmt = if json { OutputFormat::Json } else { OutputFormat::Table };
                output::render(&transitions, fmt);
                return Ok(());
            }

            let status = status.ok_or_else(|| {
                JiraError::Config("--status is required (or use --list to see available transitions)".to_string())
            })?;

            let tid = transitions.iter()
                .find(|t| t.name == status || t.to.as_ref().map(|s| s.name.as_str()) == Some(&status))
                .map(|t| t.id.clone())
                .ok_or_else(|| {
                    JiraError::Config(format!("transition '{}' not found; use --list to see available transitions", status))
                })?;

            jira_api::do_transition(&client, &key, &tid)?;
            println!("Transitioned {} to '{}'", key, status);
            Ok(())
        }
    }
}

fn run_project(cmd: ProjectCommands) -> Result<(), JiraError> {
    match cmd {
        ProjectCommands::List { json, insecure } => {
            let client = new_client(insecure)?;
            let projects = jira_api::list_projects(&client)?;
            let fmt = if json { OutputFormat::Json } else { OutputFormat::Table };
            output::render(&projects, fmt);
            Ok(())
        }
    }
}

fn run_board(cmd: BoardCommands) -> Result<(), JiraError> {
    match cmd {
        BoardCommands::List { project, json, insecure } => {
            let client = new_client(insecure)?;
            let boards = jira_api::list_boards(&client, &project)?;
            let fmt = if json { OutputFormat::Json } else { OutputFormat::Table };
            output::render(&boards.values, fmt);
            Ok(())
        }
    }
}

fn run_sprint(cmd: SprintCommands) -> Result<(), JiraError> {
    match cmd {
        SprintCommands::List { board, json, insecure } => {
            if board == 0 {
                return Err(JiraError::Config("--board is required".to_string()));
            }
            let client = new_client(insecure)?;
            let sprints = jira_api::list_sprints(&client, board)?;
            let fmt = if json { OutputFormat::Json } else { OutputFormat::Table };
            output::render(&sprints.values, fmt);
            Ok(())
        }
    }
}

// ─── Wiki Commands ──────────────────────────────────────────────────────────

fn run_wiki(cmd: WikiParent) -> Result<(), JiraError> {
    match cmd {
        WikiParent::Page { command } => run_page(command),
        WikiParent::Space { command } => run_space(command),
    }
}

fn run_page(cmd: PageCommands) -> Result<(), JiraError> {
    match cmd {
        PageCommands::View { id, json, insecure } => {
            let client = new_client(insecure)?;
            let mut page = wiki_api::get_page(&client, &id, "body.view,version,space")?;

            // Convert Confluence HTML → Markdown + download images
            if let Some(ref mut body) = page.body {
                if let Some(ref mut view) = body.view {
                    let base = page.links.as_ref().map(|l| l.base.as_str()).unwrap_or("");
                    let md = confluence::convert_to_markdown(&view.value, base);
                    let md = confluence::download_images(&md, &id, &client);
                    view.value = md;
                }
            }

            if json {
                output::render_json(&page);
                return Ok(());
            }

            // Build full markdown output
            let mut full_md = String::new();
            full_md.push_str(&format!("# {}\n\n", page.title));

            let mut meta = format!("**ID:** {}", page.id);
            if let Some(ref s) = page.space {
                meta.push_str(&format!("  |  **Space:** {}", s.key));
            }
            if let Some(ref v) = page.version {
                meta.push_str(&format!("  |  **Version:** {}", v.number));
            }
            if let Some(ref l) = page.links {
                if !l.base.is_empty() && !l.web_ui.is_empty() {
                    meta.push_str(&format!("  |  **URL:** {}{}", l.base, l.web_ui));
                }
            }
            full_md.push_str(&meta);
            full_md.push_str("\n\n---\n\n");

            if let Some(ref body) = page.body {
                if let Some(ref view) = body.view {
                    full_md.push_str(&view.value);
                }
            }

            // Print to stdout
            println!("{}", full_md);

            // Auto-save to .prd-reviewer/prd/<title>.md if inside a project
            save_prd(&page.title, &full_md);

            Ok(())
        }

        PageCommands::Search { title, cql, space, max_results, json, insecure } => {
            let client = new_client(insecure)?;
            let result = wiki_api::search_pages(&client, &title, &cql, &space, max_results)?;
            let fmt = if json { OutputFormat::Json } else { OutputFormat::Table };
            output::render(&result.results, fmt);
            Ok(())
        }

        PageCommands::Create { space, title, body, parent, insecure } => {
            let client = new_client(insecure)?;
            let mut ancestors = Vec::new();
            if !parent.is_empty() {
                ancestors.push(WikiAncestor { id: parent });
            }
            let payload = WikiCreatePage {
                page_type: "page".to_string(),
                title: title.clone(),
                space: WikiSpaceRef { key: space },
                body: WikiBodyWrite {
                    storage: WikiStorage { value: body },
                },
                ancestors,
            };
            let page = wiki_api::create_page(&client, &payload)?;
            println!("Created page: {} (ID: {})", page.title, page.id);
            Ok(())
        }

        PageCommands::Update { id, title, body, insecure } => {
            let client = new_client(insecure)?;
            let current = wiki_api::get_page(&client, &id, "version")?;
            let new_title = title.unwrap_or(current.title);
            let new_version = current.version.map(|v| v.number + 1).unwrap_or(1);
            let payload = WikiUpdatePage {
                page_type: "page".to_string(),
                title: new_title,
                version: WikiVersionWrite { number: new_version },
                body: WikiBodyWrite {
                    storage: WikiStorage {
                        value: body.unwrap_or_default(),
                    },
                },
            };
            let page = wiki_api::update_page(&client, &id, &payload)?;
            println!("Updated page: {} (ID: {})", page.title, page.id);
            Ok(())
        }

        PageCommands::Comment { id, body, file, insecure } => {
            let client = new_client(insecure)?;
            let html = if !file.is_empty() {
                std::fs::read_to_string(&file)?
            } else if !body.is_empty() {
                body
            } else {
                return Err(JiraError::Config("Provide --body or --file".to_string()));
            };
            wiki_api::add_comment(&client, &id, &html)?;
            println!("Comment added to wiki page {}", id);
            Ok(())
        }
        PageCommands::Export { id, format, output, insecure } => {
            let client = new_client(insecure)?;
            wiki_api::export_page(&client, &id, &format, &output)
        }
    }
}

fn run_space(cmd: SpaceCommands) -> Result<(), JiraError> {
    match cmd {
        SpaceCommands::List { json, insecure } => {
            let client = new_client(insecure)?;
            let spaces = wiki_api::list_spaces(&client)?;
            let fmt = if json { OutputFormat::Json } else { OutputFormat::Table };
            output::render(&spaces, fmt);
            Ok(())
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn new_client(insecure: bool) -> Result<Client, JiraError> {
    let cfg = Config::load()?;
    cfg.validate()?;
    Ok(Client::new(&cfg, insecure))
}

/// Save Jira issue markdown to .prd-reviewer/tasks/<key>.md in the current project.
fn save_task(key: &str, content: &str) {
    let project_root = match find_project_root() {
        Some(root) => root,
        None => return,
    };

    let tasks_dir = project_root.join(".prd-reviewer").join("tasks");
    if std::fs::create_dir_all(&tasks_dir).is_err() {
        return;
    }

    let filename = format!("{}.md", key);
    let filepath = tasks_dir.join(&filename);

    match std::fs::write(&filepath, content) {
        Ok(()) => eprintln!("  [saved] {}", filepath.display()),
        Err(e) => eprintln!("  [warning] Failed to save task: {}", e),
    }
}

/// Save wiki page markdown to .prd-reviewer/prd/<title>.md in the current project.
fn save_prd(title: &str, content: &str) {
    let project_root = match find_project_root() {
        Some(root) => root,
        None => return, // not inside a project, skip silently
    };

    let prd_dir = project_root.join(".prd-reviewer").join("prd");
    if std::fs::create_dir_all(&prd_dir).is_err() {
        return;
    }

    // Sanitize title for filename: replace slashes, colons, etc.
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
        Ok(()) => {
            eprintln!("  [saved] {}", filepath.display());
        }
        Err(e) => {
            eprintln!("  [warning] Failed to save PRD: {}", e);
        }
    }
}

/// Walk up from cwd to find a project root (has .prd-reviewer/ or .claude/).
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
