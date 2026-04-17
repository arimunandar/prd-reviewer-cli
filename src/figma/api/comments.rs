use crate::figma::client::Client;
use crate::figma::error::FigmaError;
use crate::figma::models::comment::{Comment, CommentsResponse, CreateCommentRequest};

pub fn get_comments(client: &Client, file_key: &str) -> Result<Vec<Comment>, FigmaError> {
    let url = format!("{}/v1/files/{}/comments", client.base_url, file_key);
    let result: CommentsResponse = client.get(&url)?;
    Ok(result.comments)
}

pub fn create_comment(
    client: &Client,
    file_key: &str,
    message: &str,
) -> Result<Comment, FigmaError> {
    let url = format!("{}/v1/files/{}/comments", client.base_url, file_key);
    let payload = CreateCommentRequest {
        message: message.to_string(),
        comment_id: None,
    };
    client.post(&url, &payload)
}

pub fn reply_to_comment(
    client: &Client,
    file_key: &str,
    comment_id: &str,
    message: &str,
) -> Result<Comment, FigmaError> {
    let url = format!("{}/v1/files/{}/comments", client.base_url, file_key);
    let payload = CreateCommentRequest {
        message: message.to_string(),
        comment_id: Some(comment_id.to_string()),
    };
    client.post(&url, &payload)
}
