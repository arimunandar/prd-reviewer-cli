use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::figma::api;
use crate::figma::client::Client;
use crate::figma::config::Config;
use crate::figma::error::FigmaError;
use crate::figma::output::{self, AsciiTreeOptions, OutputFormat};
use crate::figma::url;

#[derive(Args)]
pub struct OutputArgs {
    /// Output in JSON format
    #[arg(long)]
    json: bool,
    /// Output in YAML format
    #[arg(long)]
    yaml: bool,
}

impl OutputArgs {
    fn format(&self) -> OutputFormat {
        OutputFormat::from_flags(self.json, self.yaml)
    }
}

#[derive(Subcommand)]
pub enum FigmaCommands {
    /// Fetch node and children from a Figma URL (JSON default)
    Url {
        /// Figma URL with node-id
        url: String,
        /// Output as ASCII tree instead of JSON
        #[arg(long)]
        tree: bool,
        /// Max depth for ASCII tree output (-1 for unlimited)
        #[arg(long, default_value = "-1")]
        tree_depth: i32,
        /// Depth of node tree to fetch (-1 for unlimited)
        #[arg(long, default_value = "-1")]
        depth: i32,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// View a Figma file or node (table default)
    View {
        /// Figma URL or file key
        input: String,
        /// Node ID (overrides node-id from URL)
        #[arg(long)]
        node: Option<String>,
        /// Depth of node tree to fetch
        #[arg(long, default_value = "1")]
        depth: i32,
        /// Export format: png, svg, pdf, jpg (triggers export)
        #[arg(long)]
        format: Option<String>,
        /// Export scale (used with --format)
        #[arg(long, default_value = "1")]
        scale: f64,
        /// Output directory for export
        #[arg(long, default_value = "./")]
        output_dir: String,
        /// Output as ASCII tree
        #[arg(long)]
        ascii: bool,
        /// Max depth for ASCII tree (-1 for unlimited)
        #[arg(long, default_value = "-1")]
        tree_depth: i32,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// File operations
    File {
        #[command(subcommand)]
        command: FileCommands,
    },
    /// Node operations
    Node {
        #[command(subcommand)]
        command: NodeCommands,
    },
    /// Comment operations
    Comment {
        #[command(subcommand)]
        command: CommentCommands,
    },
    /// Team operations
    Team {
        #[command(subcommand)]
        command: TeamCommands,
    },
    /// Project operations
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    /// Variable operations
    Variable {
        #[command(subcommand)]
        command: VariableCommands,
    },
    /// Download a Figma node as PNG screenshot (token-efficient, no JSON output)
    Screenshot {
        /// Figma URL with node-id
        url: String,
        /// Export scale (default 2x)
        #[arg(long, default_value = "2")]
        scale: f64,
    },
}

#[derive(Subcommand)]
pub enum FileCommands {
    /// View file details and pages
    View {
        /// File key or Figma URL
        input: String,
        /// Depth of node tree to fetch (default 1 for pages only)
        #[arg(long, default_value = "1")]
        depth: i32,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Component operations
    Component {
        #[command(subcommand)]
        command: FileComponentCommands,
    },
    /// Style operations
    Style {
        #[command(subcommand)]
        command: FileStyleCommands,
    },
}

#[derive(Subcommand)]
pub enum FileComponentCommands {
    /// List components in a file
    List {
        /// File key or Figma URL
        input: String,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Subcommand)]
pub enum FileStyleCommands {
    /// List styles in a file
    List {
        /// File key or Figma URL
        input: String,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Subcommand)]
pub enum NodeCommands {
    /// View node details
    View {
        /// File key or Figma URL
        input: String,
        /// Node ID (or use URL with node-id)
        #[arg(long)]
        node: Option<String>,
        /// Output as ASCII tree
        #[arg(long)]
        ascii: bool,
        /// Max depth for ASCII tree (-1 for unlimited)
        #[arg(long, default_value = "-1")]
        tree_depth: i32,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Export node as image
    Export {
        /// File key or Figma URL
        input: String,
        /// Node ID (or use URL with node-id)
        #[arg(long)]
        node: Option<String>,
        /// Export format: png, svg, pdf, jpg
        #[arg(long, default_value = "png")]
        format: String,
        /// Export scale (e.g. 2 for 2x)
        #[arg(long, default_value = "1")]
        scale: f64,
        /// Output directory
        #[arg(long, default_value = "./")]
        output_dir: String,
    },
}

#[derive(Subcommand)]
pub enum CommentCommands {
    /// List comments on a file
    List {
        /// File key or Figma URL
        input: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Create a comment on a file
    Create {
        /// File key or Figma URL
        input: String,
        /// Comment message
        #[arg(long)]
        message: String,
    },
    /// Reply to a comment
    Reply {
        /// File key or Figma URL
        input: String,
        /// Parent comment ID
        #[arg(long)]
        comment: String,
        /// Reply message
        #[arg(long)]
        message: String,
    },
}

#[derive(Subcommand)]
pub enum TeamCommands {
    /// Project operations under a team
    Project {
        #[command(subcommand)]
        command: TeamProjectCommands,
    },
}

#[derive(Subcommand)]
pub enum TeamProjectCommands {
    /// List projects in a team
    List {
        /// Team ID
        #[arg(long)]
        team: Option<String>,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// File operations under a project
    File {
        #[command(subcommand)]
        command: ProjectFileCommands,
    },
}

#[derive(Subcommand)]
pub enum ProjectFileCommands {
    /// List files in a project
    List {
        /// Project ID
        project_id: String,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Subcommand)]
pub enum VariableCommands {
    /// List variables in a file
    List {
        /// File key or Figma URL
        input: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Variable collection operations
    Collection {
        #[command(subcommand)]
        command: VariableCollectionCommands,
    },
}

#[derive(Subcommand)]
pub enum VariableCollectionCommands {
    /// List variable collections in a file
    List {
        /// File key or Figma URL
        input: String,
        #[command(flatten)]
        output: OutputArgs,
    },
}

// ─── Dispatch ───────────────────────────────────────────────────────────────

pub fn run(cmd: FigmaCommands) {
    if let Err(e) = run_inner(cmd) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_inner(cmd: FigmaCommands) -> Result<(), FigmaError> {
    match cmd {
        FigmaCommands::Url {
            url: url_str,
            tree,
            tree_depth,
            depth: _,
            output: out,
        } => run_url(&url_str, tree, tree_depth, &out),
        FigmaCommands::View {
            input,
            node,
            depth,
            format,
            scale,
            output_dir,
            ascii,
            tree_depth,
            output: out,
        } => run_view(&input, node.as_deref(), depth, format.as_deref(), scale, &output_dir, ascii, tree_depth, &out),
        FigmaCommands::File { command } => run_file(command),
        FigmaCommands::Node { command } => run_node(command),
        FigmaCommands::Comment { command } => run_comment(command),
        FigmaCommands::Team { command } => run_team(command),
        FigmaCommands::Project { command } => run_project(command),
        FigmaCommands::Variable { command } => run_variable(command),
        FigmaCommands::Screenshot { url: url_str, scale } => run_screenshot(&url_str, scale),
    }
}

// ─── Url ────────────────────────────────────────────────────────────────────

fn run_url(url_str: &str, tree: bool, tree_depth: i32, out: &OutputArgs) -> Result<(), FigmaError> {
    let parsed = url::parse_detailed(url_str)?;

    // If no node ID, just output the parsed URL info
    let node_api = parsed.node.as_ref().map(|n| n.api_format.clone());
    if node_api.is_none() || node_api.as_deref() == Some("") {
        if out.yaml {
            output::render_yaml(&parsed);
        } else {
            output::render_json(&parsed);
        }
        return Ok(());
    }
    let node_id = node_api.unwrap();

    let client = new_client()?;
    let result = api::files::get_file_nodes(&client, &parsed.file_key, &[&node_id])?;

    if tree {
        for detail in result.nodes.values() {
            let opts = AsciiTreeOptions {
                max_depth: tree_depth,
                show_id: true,
            };
            output::render_ascii_tree(&detail.document, &opts);
        }
        return Ok(());
    }

    // Fetch screenshot
    let screenshot = fetch_screenshot(&client, &parsed.file_key, &node_id);

    #[derive(serde::Serialize)]
    struct UrlResponse {
        #[serde(skip_serializing_if = "Option::is_none")]
        screenshot: Option<String>,
        nodes: std::collections::HashMap<String, crate::figma::models::file::NodeDetail>,
    }

    let resp = UrlResponse {
        screenshot: if screenshot.is_empty() { None } else { Some(screenshot) },
        nodes: result.nodes,
    };

    if out.yaml {
        output::render_yaml(&resp);
    } else {
        output::render_json(&resp);
    }
    Ok(())
}

fn fetch_screenshot(client: &Client, file_key: &str, node_id: &str) -> String {
    let img_resp = match api::images::get_image_urls(client, file_key, &[node_id], "png", 2.0) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Warning: could not fetch screenshot URL: {}", e);
            return String::new();
        }
    };
    let img_url = match img_resp.images.get(node_id) {
        Some(Some(u)) if !u.is_empty() => u,
        _ => {
            eprintln!("Warning: no screenshot URL returned for node {}", node_id);
            return String::new();
        }
    };

    let tmp_dir = std::env::temp_dir();
    let safe_id = node_id.replace(':', "-");
    let filename = format!("figma_{}_{}.png", file_key, safe_id);
    let out_path = tmp_dir.join(filename);
    let out_path_str = out_path.to_string_lossy().to_string();

    if let Err(e) = api::images::download_image(img_url, &out_path_str) {
        eprintln!("Warning: could not download screenshot: {}", e);
        return String::new();
    }

    out_path_str
}

// ─── Screenshot ────────────────────────────────────────────────────────────

fn run_screenshot(url_str: &str, scale: f64) -> Result<(), FigmaError> {
    let parsed = url::parse(url_str)?;

    if parsed.file_key.is_empty() {
        return Err(FigmaError::Parse("Could not extract file key from URL".to_string()));
    }
    if parsed.node_id.is_empty() {
        return Err(FigmaError::Parse("Could not extract node-id from URL. Add ?node-id=X-Y to the URL.".to_string()));
    }

    let cfg = Config::load()?;
    let client = Client::new(&cfg);

    let img_resp = api::images::get_image_urls(
        &client,
        &parsed.file_key,
        &[parsed.node_id.as_str()],
        "png",
        scale,
    )?;

    let img_url = img_resp
        .images
        .get(&parsed.node_id)
        .and_then(|u| u.as_ref())
        .filter(|u| !u.is_empty())
        .ok_or_else(|| FigmaError::Api { status: 0, body: "No image URL returned for this node".to_string() })?;

    let safe_id = parsed.node_id.replace(':', "-");
    let filename = format!("figma_{}_{}_{}x.png", parsed.file_key, safe_id, scale as u32);

    // Save to .prd-reviewer/figma/ if project root exists, otherwise temp dir
    let out_path = find_figma_output_dir()
        .map(|dir| dir.join(&filename))
        .unwrap_or_else(|| std::env::temp_dir().join(&filename));

    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let out_path_str = out_path.to_string_lossy().to_string();
    api::images::download_image(img_url, &out_path_str)?;

    println!("{}", out_path_str);
    Ok(())
}

fn find_figma_output_dir() -> Option<std::path::PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join(".prd-reviewer").exists() || dir.join(".claude").exists() {
            return Some(dir.join(".prd-reviewer").join("figma"));
        }
        if !dir.pop() {
            return None;
        }
    }
}

// ─── View ───────────────────────────────────────────────────────────────────

fn run_view(
    input: &str,
    node_flag: Option<&str>,
    depth: i32,
    format: Option<&str>,
    scale: f64,
    output_dir: &str,
    ascii: bool,
    tree_depth: i32,
    out: &OutputArgs,
) -> Result<(), FigmaError> {
    let parsed = url::parse(input)?;
    let node_id = node_flag
        .map(|s| s.to_string())
        .or_else(|| {
            if parsed.node_id.is_empty() {
                None
            } else {
                Some(parsed.node_id.clone())
            }
        });

    let client = new_client()?;

    // Export mode
    if let Some(fmt) = format {
        let nid = node_id.ok_or_else(|| {
            FigmaError::Parse("--node or a URL with node-id is required for export".to_string())
        })?;
        return do_export(&client, &parsed.file_key, &nid, fmt, scale, output_dir);
    }

    // Node view
    if let Some(nid) = node_id {
        return do_view_node(&client, &parsed.file_key, &nid, ascii, tree_depth, out);
    }

    // File view
    do_view_file(&client, &parsed.file_key, depth, out)
}

fn do_view_file(client: &Client, file_key: &str, depth: i32, out: &OutputArgs) -> Result<(), FigmaError> {
    let file = api::files::get_file(client, file_key, depth)?;

    match out.format() {
        OutputFormat::Json => { output::render_json(&file); }
        OutputFormat::Yaml => { output::render_yaml(&file); }
        OutputFormat::Table => {
            println!("# {}\n", file.name);
            println!("| Field | Value |");
            println!("| --- | --- |");
            println!("| Version | {} |", file.version);
            println!("| Last Modified | {} |", file.last_modified);
            println!("| Role | {} |", file.role);
            println!("| Schema Version | {} |", file.schema_version);
            println!();
            println!("## Pages");
            output::render_table(&file.document.children);
        }
    }
    Ok(())
}

fn do_view_node(
    client: &Client,
    file_key: &str,
    node_id: &str,
    ascii: bool,
    tree_depth: i32,
    out: &OutputArgs,
) -> Result<(), FigmaError> {
    let result = api::files::get_file_nodes(client, file_key, &[node_id])?;

    match out.format() {
        OutputFormat::Json => { output::render_json(&result); }
        OutputFormat::Yaml => { output::render_yaml(&result); }
        OutputFormat::Table => {
            for detail in result.nodes.values() {
                let node = &detail.document;

                if ascii {
                    let opts = AsciiTreeOptions {
                        max_depth: tree_depth,
                        show_id: false,
                    };
                    output::render_ascii_tree(node, &opts);
                    return Ok(());
                }

                println!("# {}\n", node.name);
                println!("| Field | Value |");
                println!("| --- | --- |");
                println!("| ID | {} |", node_id);
                println!("| Type | {} |", node.node_type);
                if let Some(ref bb) = node.absolute_bounding_box {
                    println!("| Position | x={:.0}, y={:.0} |", bb.x, bb.y);
                    println!("| Size | {:.0} x {:.0} |", bb.width, bb.height);
                }
                if !node.characters.is_empty() {
                    println!("| Text | {} |", node.characters);
                }
                if let Some(ref style) = node.style {
                    println!("| Font | {} {:.0} |", style.font_family, style.font_size);
                }
                println!();

                if !node.children.is_empty() {
                    println!("## Children");
                    output::render_table(&node.children);
                }
            }
        }
    }
    Ok(())
}

fn do_export(
    client: &Client,
    file_key: &str,
    node_id: &str,
    format: &str,
    scale: f64,
    output_dir: &str,
) -> Result<(), FigmaError> {
    let result = api::images::get_image_urls(client, file_key, &[node_id], format, scale)?;
    if !result.err.is_empty() {
        return Err(FigmaError::Api {
            status: 0,
            body: format!("Figma API error: {}", result.err),
        });
    }

    for (id, image_url) in &result.images {
        let url = match image_url {
            Some(u) if !u.is_empty() => u,
            _ => {
                println!("No image generated for node {}", id);
                continue;
            }
        };
        let safe_id = id.replace(':', "-");
        let filename = format!("{}_{:.0}x.{}", safe_id, scale, format);
        let output_path = PathBuf::from(output_dir).join(&filename);
        let output_path_str = output_path.to_string_lossy().to_string();

        println!("Downloading {}...", filename);
        api::images::download_image(url, &output_path_str)?;
        println!("Saved to {}", output_path_str);
    }
    Ok(())
}

// ─── File ───────────────────────────────────────────────────────────────────

fn run_file(cmd: FileCommands) -> Result<(), FigmaError> {
    match cmd {
        FileCommands::View {
            input,
            depth,
            output: out,
        } => {
            let client = new_client()?;
            let fk = url::file_key(&input);
            do_view_file(&client, &fk, depth, &out)
        }
        FileCommands::Component { command } => match command {
            FileComponentCommands::List { input, output: out } => {
                let client = new_client()?;
                let components = api::files::get_file_components(&client, &url::file_key(&input))?;
                output::render(&components, out.format());
                Ok(())
            }
        },
        FileCommands::Style { command } => match command {
            FileStyleCommands::List { input, output: out } => {
                let client = new_client()?;
                let styles = api::files::get_file_styles(&client, &url::file_key(&input))?;
                output::render(&styles, out.format());
                Ok(())
            }
        },
    }
}

// ─── Node ───────────────────────────────────────────────────────────────────

fn run_node(cmd: NodeCommands) -> Result<(), FigmaError> {
    match cmd {
        NodeCommands::View {
            input,
            node,
            ascii,
            tree_depth,
            output: out,
        } => {
            let parsed = url::parse(&input)?;
            let node_id = node
                .or_else(|| {
                    if parsed.node_id.is_empty() {
                        None
                    } else {
                        Some(parsed.node_id.clone())
                    }
                })
                .ok_or_else(|| {
                    FigmaError::Parse("--node is required (or pass a URL with node-id)".to_string())
                })?;
            let client = new_client()?;
            do_view_node(&client, &parsed.file_key, &node_id, ascii, tree_depth, &out)
        }
        NodeCommands::Export {
            input,
            node,
            format,
            scale,
            output_dir,
        } => {
            let parsed = url::parse(&input)?;
            let node_id = node
                .or_else(|| {
                    if parsed.node_id.is_empty() {
                        None
                    } else {
                        Some(parsed.node_id.clone())
                    }
                })
                .ok_or_else(|| {
                    FigmaError::Parse("--node is required (or pass a URL with node-id)".to_string())
                })?;
            let client = new_client()?;
            do_export(&client, &parsed.file_key, &node_id, &format, scale, &output_dir)
        }
    }
}

// ─── Comment ────────────────────────────────────────────────────────────────

fn run_comment(cmd: CommentCommands) -> Result<(), FigmaError> {
    match cmd {
        CommentCommands::List { input, output: out } => {
            let client = new_client()?;
            let comments = api::comments::get_comments(&client, &url::file_key(&input))?;
            output::render(&comments, out.format());
            Ok(())
        }
        CommentCommands::Create { input, message } => {
            let client = new_client()?;
            let comment = api::comments::create_comment(&client, &url::file_key(&input), &message)?;
            println!("Comment created (ID: {})", comment.id);
            Ok(())
        }
        CommentCommands::Reply {
            input,
            comment,
            message,
        } => {
            let client = new_client()?;
            let reply = api::comments::reply_to_comment(
                &client,
                &url::file_key(&input),
                &comment,
                &message,
            )?;
            println!("Reply created (ID: {})", reply.id);
            Ok(())
        }
    }
}

// ─── Team ───────────────────────────────────────────────────────────────────

fn run_team(cmd: TeamCommands) -> Result<(), FigmaError> {
    match cmd {
        TeamCommands::Project { command } => match command {
            TeamProjectCommands::List { team, output: out } => {
                let client = new_client()?;
                let team_id = team.or_else(|| {
                    Config::load().ok().and_then(|c| {
                        if c.default_team_id.is_empty() {
                            None
                        } else {
                            Some(c.default_team_id)
                        }
                    })
                });
                let team_id = team_id.ok_or_else(|| {
                    FigmaError::Config(
                        "--team is required (or set default_team_id in config)".to_string(),
                    )
                })?;
                let projects = api::teams::get_team_projects(&client, &team_id)?;
                output::render(&projects, out.format());
                Ok(())
            }
        },
    }
}

// ─── Project ────────────────────────────────────────────────────────────────

fn run_project(cmd: ProjectCommands) -> Result<(), FigmaError> {
    match cmd {
        ProjectCommands::File { command } => match command {
            ProjectFileCommands::List {
                project_id,
                output: out,
            } => {
                let client = new_client()?;
                let files = api::teams::get_project_files(&client, &project_id)?;
                output::render(&files, out.format());
                Ok(())
            }
        },
    }
}

// ─── Variable ───────────────────────────────────────────────────────────────

fn run_variable(cmd: VariableCommands) -> Result<(), FigmaError> {
    match cmd {
        VariableCommands::List { input, output: out } => {
            let client = new_client()?;
            let meta = api::variables::get_local_variables(&client, &url::file_key(&input))?;

            if matches!(out.format(), OutputFormat::Json) {
                output::render_json(&meta.variables);
                return Ok(());
            }

            let collection_names: std::collections::HashMap<String, String> = meta
                .variable_collections
                .iter()
                .map(|(id, col)| (id.clone(), col.name.clone()))
                .collect();

            let rows: Vec<crate::figma::models::variable::VariableTableRow> = meta
                .variables
                .values()
                .map(|v| crate::figma::models::variable::VariableTableRow {
                    id: v.id.clone(),
                    name: v.name.clone(),
                    resolved_type: v.resolved_type.clone(),
                    collection_name: collection_names
                        .get(&v.variable_collection_id)
                        .cloned()
                        .unwrap_or_default(),
                    description: v.description.clone(),
                })
                .collect();

            output::render(&rows, out.format());
            Ok(())
        }
        VariableCommands::Collection { command } => match command {
            VariableCollectionCommands::List { input, output: out } => {
                let client = new_client()?;
                let meta = api::variables::get_local_variables(&client, &url::file_key(&input))?;

                if matches!(out.format(), OutputFormat::Json) {
                    output::render_json(&meta.variable_collections);
                    return Ok(());
                }

                let collections: Vec<crate::figma::models::variable::VariableCollection> =
                    meta.variable_collections.into_values().collect();
                output::render(&collections, out.format());
                Ok(())
            }
        },
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn new_client() -> Result<Client, FigmaError> {
    let cfg = Config::load()?;
    cfg.validate()?;
    Ok(Client::new(&cfg))
}
