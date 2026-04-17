use crate::figma::output::Tableable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub message: String,
    pub file_key: String,
    pub parent_id: String,
    pub user: CommentUser,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub resolved_at: String,
    pub order_id: String,
}

impl Tableable for Comment {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "User", "Message", "Created", "Resolved", "Parent"]
    }
    fn row(&self) -> Vec<String> {
        let msg = if self.message.len() > 60 {
            format!("{}...", &self.message[..60])
        } else {
            self.message.clone()
        };
        let resolved = if self.resolved_at.is_empty() {
            String::new()
        } else {
            "yes".to_string()
        };
        vec![
            self.id.clone(),
            self.user.handle.clone(),
            msg,
            self.created_at.clone(),
            resolved,
            self.parent_id.clone(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentUser {
    pub handle: String,
    pub img_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentsResponse {
    pub comments: Vec<Comment>,
}

#[derive(Debug, Serialize)]
pub struct CreateCommentRequest {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<String>,
}
