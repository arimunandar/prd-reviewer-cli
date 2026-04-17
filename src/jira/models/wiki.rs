use crate::figma::output::Tableable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiPage {
    pub id: String,
    #[serde(rename = "type", default)]
    pub page_type: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: String,
    pub version: Option<WikiVersion>,
    pub body: Option<WikiBody>,
    #[serde(rename = "_links")]
    pub links: Option<WikiLinks>,
    pub space: Option<WikiSpace>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiVersion {
    pub number: i32,
    pub by: Option<WikiUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiUser {
    #[serde(rename = "displayName", default)]
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiBody {
    pub storage: Option<WikiStorage>,
    pub view: Option<WikiStorage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiStorage {
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiLinks {
    #[serde(default)]
    pub base: String,
    #[serde(rename = "webui", default)]
    pub web_ui: String,
    #[serde(rename = "tinyui", default)]
    pub tiny_ui: String,
}

impl Tableable for WikiPage {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Title", "Space", "Status", "Version"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.title.clone(),
            self.space.as_ref().map(|s| s.key.clone()).unwrap_or_default(),
            self.status.clone(),
            self.version.as_ref().map(|v| v.number.to_string()).unwrap_or_default(),
        ]
    }
}

// Search
#[derive(Debug, Serialize, Deserialize)]
pub struct WikiSearchResult {
    pub results: Vec<WikiPage>,
    #[serde(default)]
    pub size: i32,
}

// Space
#[derive(Debug, Serialize, Deserialize)]
pub struct WikiSpaceList {
    pub results: Vec<WikiSpace>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiSpace {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type", default)]
    pub space_type: String,
}

impl Tableable for WikiSpace {
    fn headers() -> Vec<&'static str> {
        vec!["Key", "Name", "Type"]
    }
    fn row(&self) -> Vec<String> {
        vec![self.key.clone(), self.name.clone(), self.space_type.clone()]
    }
}

// Create/Update payloads
#[derive(Debug, Serialize)]
pub struct WikiCreatePage {
    #[serde(rename = "type")]
    pub page_type: String,
    pub title: String,
    pub space: WikiSpaceRef,
    pub body: WikiBodyWrite,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ancestors: Vec<WikiAncestor>,
}

#[derive(Debug, Serialize)]
pub struct WikiSpaceRef {
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct WikiBodyWrite {
    pub storage: WikiStorage,
}

#[derive(Debug, Serialize)]
pub struct WikiAncestor {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct WikiUpdatePage {
    #[serde(rename = "type")]
    pub page_type: String,
    pub title: String,
    pub version: WikiVersionWrite,
    pub body: WikiBodyWrite,
}

#[derive(Debug, Serialize)]
pub struct WikiVersionWrite {
    pub number: i32,
}
