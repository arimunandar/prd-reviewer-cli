use crate::figma::client::Client;
use crate::figma::error::FigmaError;
use crate::figma::models::variable::{VariablesMeta, VariablesResponse};

pub fn get_local_variables(
    client: &Client,
    file_key: &str,
) -> Result<VariablesMeta, FigmaError> {
    let url = format!(
        "{}/v1/files/{}/variables/local",
        client.base_url, file_key
    );
    let result: VariablesResponse = client.get(&url)?;
    Ok(result.meta)
}
