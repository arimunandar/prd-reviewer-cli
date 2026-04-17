use crate::figma::output::Tableable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileResponse {
    pub name: String,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: String,
    pub version: String,
    pub role: String,
    pub document: DocumentNode,
    #[serde(rename = "schemaVersion")]
    pub schema_version: i32,
}

impl Tableable for FileResponse {
    fn headers() -> Vec<&'static str> {
        vec!["Name", "Last Modified", "Version", "Role"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.last_modified.clone(),
            self.version.clone(),
            self.role.clone(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default)]
    pub children: Vec<PageNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default)]
    pub children: Vec<NodeInfo>,
}

impl Tableable for PageNode {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Type", "Children"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.name.clone(),
            self.node_type.clone(),
            self.children.len().to_string(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<NodeInfo>,
    #[serde(rename = "blendMode", default, skip_serializing_if = "String::is_empty")]
    pub blend_mode: String,
    #[serde(rename = "absoluteBoundingBox", skip_serializing_if = "Option::is_none")]
    pub absolute_bounding_box: Option<Rectangle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fills: Vec<Paint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strokes: Vec<Paint>,
    #[serde(rename = "strokeWeight", default)]
    pub stroke_weight: f64,
    #[serde(rename = "cornerRadius", default)]
    pub corner_radius: f64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub characters: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<TypeStyle>,
}

impl Tableable for NodeInfo {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Type", "Visible", "Children"]
    }
    fn row(&self) -> Vec<String> {
        let visible = match self.visible {
            Some(false) => "false",
            _ => "true",
        };
        vec![
            self.id.clone(),
            self.name.clone(),
            self.node_type.clone(),
            visible.to_string(),
            self.children.len().to_string(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paint {
    #[serde(rename = "type")]
    pub paint_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(default)]
    pub opacity: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TypeStyle {
    #[serde(rename = "fontFamily")]
    pub font_family: String,
    #[serde(rename = "fontWeight")]
    pub font_weight: f64,
    #[serde(rename = "fontSize")]
    pub font_size: f64,
    #[serde(rename = "lineHeightPx")]
    pub line_height_px: f64,
    #[serde(rename = "letterSpacing")]
    pub letter_spacing: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileNodesResponse {
    pub nodes: HashMap<String, NodeDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeDetail {
    pub document: NodeInfo,
}
