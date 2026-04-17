use crate::figma::client::Client;
use crate::figma::error::FigmaError;
use crate::figma::models::team::{
    Project, ProjectFile, ProjectFilesResponse, ProjectsResponse,
};

pub fn get_team_projects(client: &Client, team_id: &str) -> Result<Vec<Project>, FigmaError> {
    let url = format!("{}/v1/teams/{}/projects", client.base_url, team_id);
    let result: ProjectsResponse = client.get(&url)?;
    Ok(result.projects)
}

pub fn get_project_files(
    client: &Client,
    project_id: &str,
) -> Result<Vec<ProjectFile>, FigmaError> {
    let url = format!("{}/v1/projects/{}/files", client.base_url, project_id);
    let result: ProjectFilesResponse = client.get(&url)?;
    Ok(result.files)
}
