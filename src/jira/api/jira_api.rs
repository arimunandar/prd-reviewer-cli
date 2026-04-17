use crate::jira::client::Client;
use crate::jira::error::JiraError;
use crate::jira::models::jira::*;

pub fn get_issue(client: &Client, key: &str, expand: &str) -> Result<JiraIssue, JiraError> {
    let mut url = format!("{}/issue/{}", client.base_jira, key);
    if !expand.is_empty() {
        url.push_str(&format!("?expand={}", expand));
    }
    client.get(&url)
}

pub fn search_issues(
    client: &Client,
    jql: &str,
    max_results: i32,
) -> Result<JiraSearchResult, JiraError> {
    let url = format!(
        "{}/search?jql={}&maxResults={}",
        client.base_jira,
        url_encode(jql),
        max_results
    );
    client.get(&url)
}

pub fn create_issue(client: &Client, payload: &JiraCreateIssue) -> Result<JiraIssue, JiraError> {
    let url = format!("{}/issue", client.base_jira);
    client.post(&url, payload)
}

pub fn update_issue(client: &Client, key: &str, payload: &JiraCreateIssue) -> Result<(), JiraError> {
    let url = format!("{}/issue/{}", client.base_jira, key);
    client.put_no_response(&url, payload)
}

pub fn add_comment(client: &Client, key: &str, body: &str) -> Result<(), JiraError> {
    let url = format!("{}/issue/{}/comment", client.base_jira, key);
    let payload = JiraAddComment {
        body: body.to_string(),
    };
    client.post_no_response(&url, &payload)
}

pub fn get_transitions(
    client: &Client,
    key: &str,
) -> Result<Vec<JiraTransition>, JiraError> {
    let url = format!("{}/issue/{}/transitions", client.base_jira, key);
    let result: JiraTransitionsResponse = client.get(&url)?;
    Ok(result.transitions)
}

pub fn do_transition(client: &Client, key: &str, transition_id: &str) -> Result<(), JiraError> {
    let url = format!("{}/issue/{}/transitions", client.base_jira, key);
    let payload = JiraDoTransition {
        transition: JiraTransitionId {
            id: transition_id.to_string(),
        },
    };
    client.post_no_response(&url, &payload)
}

pub fn list_projects(client: &Client) -> Result<Vec<JiraProject>, JiraError> {
    let url = format!("{}/project", client.base_jira);
    client.get(&url)
}

pub fn list_boards(client: &Client, project_key: &str) -> Result<JiraBoardList, JiraError> {
    let mut url = format!("{}/board", client.agile_base());
    if !project_key.is_empty() {
        url.push_str(&format!("?projectKeyOrId={}", url_encode(project_key)));
    }
    client.get(&url)
}

pub fn list_sprints(client: &Client, board_id: i32) -> Result<JiraSprintList, JiraError> {
    let url = format!("{}/board/{}/sprint", client.agile_base(), board_id);
    client.get(&url)
}

fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
