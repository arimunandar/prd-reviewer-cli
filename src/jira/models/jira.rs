use crate::figma::output::Tableable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraIssue {
    pub key: String,
    pub fields: JiraIssueFields,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraIssueFields {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub description: String,
    pub status: Option<JiraStatus>,
    pub issuetype: Option<JiraIssueType>,
    pub priority: Option<JiraPriority>,
    pub assignee: Option<JiraUser>,
    pub reporter: Option<JiraUser>,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub updated: String,
    pub comment: Option<JiraCommentList>,
    #[serde(default)]
    pub attachment: Vec<JiraAttachment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraAttachment {
    pub filename: String,
    pub content: String,
    #[serde(rename = "mimeType", default)]
    pub mime_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraStatus {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraIssueType {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraPriority {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraUser {
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraCommentList {
    #[serde(default)]
    pub comments: Vec<JiraComment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraComment {
    pub author: Option<JiraUser>,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub created: String,
}

impl Tableable for JiraIssue {
    fn headers() -> Vec<&'static str> {
        vec!["Key", "Type", "Status", "Priority", "Assignee", "Summary"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.key.clone(),
            self.fields.issuetype.as_ref().map(|t| t.name.clone()).unwrap_or_default(),
            self.fields.status.as_ref().map(|s| s.name.clone()).unwrap_or_default(),
            self.fields.priority.as_ref().map(|p| p.name.clone()).unwrap_or_default(),
            self.fields.assignee.as_ref().map(|a| a.display_name.clone()).unwrap_or_default(),
            self.fields.summary.clone(),
        ]
    }
}

// Search
#[derive(Debug, Serialize, Deserialize)]
pub struct JiraSearchResult {
    pub total: i32,
    pub issues: Vec<JiraIssue>,
}

// Create/Update
#[derive(Debug, Serialize)]
pub struct JiraCreateIssue {
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct JiraAddComment {
    pub body: String,
}

// Transitions
#[derive(Debug, Serialize, Deserialize)]
pub struct JiraTransitionsResponse {
    pub transitions: Vec<JiraTransition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraTransition {
    pub id: String,
    pub name: String,
    pub to: Option<JiraStatus>,
}

impl Tableable for JiraTransition {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "To Status"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.name.clone(),
            self.to.as_ref().map(|t| t.name.clone()).unwrap_or_default(),
        ]
    }
}

#[derive(Debug, Serialize)]
pub struct JiraDoTransition {
    pub transition: JiraTransitionId,
}

#[derive(Debug, Serialize)]
pub struct JiraTransitionId {
    pub id: String,
}

// Projects
#[derive(Debug, Serialize, Deserialize)]
pub struct JiraProject {
    pub key: String,
    pub name: String,
    pub lead: Option<JiraUser>,
}

impl Tableable for JiraProject {
    fn headers() -> Vec<&'static str> {
        vec!["Key", "Name", "Lead"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.key.clone(),
            self.name.clone(),
            self.lead.as_ref().map(|l| l.display_name.clone()).unwrap_or_default(),
        ]
    }
}

// Boards
#[derive(Debug, Serialize, Deserialize)]
pub struct JiraBoardList {
    pub values: Vec<JiraBoard>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraBoard {
    pub id: i32,
    pub name: String,
    #[serde(rename = "type", default)]
    pub board_type: String,
}

impl Tableable for JiraBoard {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Type"]
    }
    fn row(&self) -> Vec<String> {
        vec![self.id.to_string(), self.name.clone(), self.board_type.clone()]
    }
}

// Sprints
#[derive(Debug, Serialize, Deserialize)]
pub struct JiraSprintList {
    pub values: Vec<JiraSprint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraSprint {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub state: String,
    #[serde(rename = "startDate", default)]
    pub start: String,
    #[serde(rename = "endDate", default)]
    pub end: String,
}

impl Tableable for JiraSprint {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "State", "Start", "End"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.name.clone(),
            self.state.clone(),
            self.start.clone(),
            self.end.clone(),
        ]
    }
}
