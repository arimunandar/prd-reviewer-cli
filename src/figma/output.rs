use crate::figma::models::file::NodeInfo;
use comfy_table::{Table, presets::UTF8_BORDERS_ONLY};
use serde::Serialize;

pub trait Tableable {
    fn headers() -> Vec<&'static str>;
    fn row(&self) -> Vec<String>;
}

#[derive(Clone, Copy)]
pub enum OutputFormat {
    Json,
    Yaml,
    Table,
}

impl OutputFormat {
    pub fn from_flags(json: bool, yaml: bool) -> OutputFormat {
        if json {
            OutputFormat::Json
        } else if yaml {
            OutputFormat::Yaml
        } else {
            OutputFormat::Table
        }
    }
}

pub fn render_json<T: Serialize + ?Sized>(data: &T) {
    let json = serde_json::to_string_pretty(data).unwrap_or_else(|e| format!("Error: {}", e));
    println!("{}", json);
}

pub fn render_yaml<T: Serialize + ?Sized>(data: &T) {
    let yaml = serde_yaml::to_string(data).unwrap_or_else(|e| format!("Error: {}", e));
    print!("{}", yaml);
}

pub fn render_table<T: Tableable>(items: &[T]) {
    if items.is_empty() {
        println!("No results found.");
        return;
    }
    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header(T::headers());
    for item in items {
        table.add_row(item.row());
    }
    println!("{}", table);
}

pub fn render<T: Tableable + Serialize>(items: &[T], format: OutputFormat) {
    match format {
        OutputFormat::Json => render_json(items),
        OutputFormat::Yaml => render_yaml(items),
        OutputFormat::Table => render_table(items),
    }
}

#[allow(dead_code)]
pub fn render_single<T: Tableable + Serialize>(item: &T, format: OutputFormat) {
    match format {
        OutputFormat::Json => render_json(item),
        OutputFormat::Yaml => render_yaml(item),
        OutputFormat::Table => render_table(&[item]),
    }
}

// Workaround: render_table expects a slice, but we need &T to work
impl<T: Tableable> Tableable for &T {
    fn headers() -> Vec<&'static str> {
        T::headers()
    }
    fn row(&self) -> Vec<String> {
        (*self).row()
    }
}

pub struct AsciiTreeOptions {
    pub max_depth: i32,
    pub show_id: bool,
}

pub fn render_ascii_tree(node: &NodeInfo, opts: &AsciiTreeOptions) {
    println!("{} ({})", node.name, node.node_type);
    let children = &node.children;
    for (i, child) in children.iter().enumerate() {
        render_tree_node(child, "", i == children.len() - 1, 1, opts);
    }
}

fn render_tree_node(
    node: &NodeInfo,
    prefix: &str,
    is_last: bool,
    depth: i32,
    opts: &AsciiTreeOptions,
) {
    if opts.max_depth >= 0 && depth > opts.max_depth {
        return;
    }

    let connector = if is_last { "\u{2514}\u{2500}\u{2500} " } else { "\u{251c}\u{2500}\u{2500} " };

    let label = if opts.show_id {
        format!("{} ({}) [{}]", node.name, node.node_type, node.id)
    } else {
        format!("{} ({})", node.name, node.node_type)
    };
    println!("{}{}{}", prefix, connector, label);

    let child_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}\u{2502}   ", prefix)
    };

    let children = &node.children;
    for (i, child) in children.iter().enumerate() {
        render_tree_node(child, &child_prefix, i == children.len() - 1, depth + 1, opts);
    }
}
