use crate::figma::output::Tableable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
}

impl Tableable for Project {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name"]
    }
    fn row(&self) -> Vec<String> {
        vec![self.id.clone(), self.name.clone()]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectsResponse {
    pub name: String,
    pub projects: Vec<Project>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    pub key: String,
    pub name: String,
    pub thumbnail_url: String,
    pub last_modified: String,
}

impl Tableable for ProjectFile {
    fn headers() -> Vec<&'static str> {
        vec!["Key", "Name", "Last Modified"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.key.clone(),
            self.name.clone(),
            self.last_modified.clone(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFilesResponse {
    pub name: String,
    pub files: Vec<ProjectFile>,
}
