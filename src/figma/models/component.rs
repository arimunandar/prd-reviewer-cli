use crate::figma::output::Tableable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentMeta {
    pub key: String,
    pub file_key: String,
    pub node_id: String,
    pub thumbnail_url: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub containing_frame: Option<FrameInfo>,
}

impl Tableable for ComponentMeta {
    fn headers() -> Vec<&'static str> {
        vec!["Key", "Name", "Node ID", "Description"]
    }
    fn row(&self) -> Vec<String> {
        let desc = if self.description.len() > 40 {
            format!("{}...", &self.description[..40])
        } else {
            self.description.clone()
        };
        vec![self.key.clone(), self.name.clone(), self.node_id.clone(), desc]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameInfo {
    pub node_id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileComponentsResponse {
    pub meta: FileComponentsMeta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileComponentsMeta {
    pub components: Vec<ComponentMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleMeta {
    pub key: String,
    pub file_key: String,
    pub node_id: String,
    pub style_type: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

impl Tableable for StyleMeta {
    fn headers() -> Vec<&'static str> {
        vec!["Key", "Name", "Type", "Node ID", "Description"]
    }
    fn row(&self) -> Vec<String> {
        let desc = if self.description.len() > 40 {
            format!("{}...", &self.description[..40])
        } else {
            self.description.clone()
        };
        vec![
            self.key.clone(),
            self.name.clone(),
            self.style_type.clone(),
            self.node_id.clone(),
            desc,
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileStylesResponse {
    pub meta: FileStylesMeta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileStylesMeta {
    pub styles: Vec<StyleMeta>,
}
