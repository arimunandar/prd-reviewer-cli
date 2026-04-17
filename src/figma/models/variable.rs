use crate::figma::output::Tableable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct VariablesResponse {
    pub meta: VariablesMeta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariablesMeta {
    pub variables: HashMap<String, Variable>,
    #[serde(rename = "variableCollections")]
    pub variable_collections: HashMap<String, VariableCollection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Variable {
    pub id: String,
    pub name: String,
    pub key: String,
    #[serde(rename = "variableCollectionId")]
    pub variable_collection_id: String,
    #[serde(rename = "resolvedType")]
    pub resolved_type: String,
    #[serde(rename = "valuesByMode")]
    pub values_by_mode: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariableTableRow {
    pub id: String,
    pub name: String,
    pub resolved_type: String,
    pub collection_name: String,
    pub description: String,
}

impl Tableable for VariableTableRow {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Type", "Collection", "Description"]
    }
    fn row(&self) -> Vec<String> {
        let desc = if self.description.len() > 40 {
            format!("{}...", &self.description[..40])
        } else {
            self.description.clone()
        };
        vec![
            self.id.clone(),
            self.name.clone(),
            self.resolved_type.clone(),
            self.collection_name.clone(),
            desc,
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VariableCollection {
    pub id: String,
    pub name: String,
    pub key: String,
    pub modes: Vec<Mode>,
    #[serde(rename = "variableIds")]
    pub variable_ids: Vec<String>,
}

impl Tableable for VariableCollection {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Name", "Key", "Modes", "Variables"]
    }
    fn row(&self) -> Vec<String> {
        let mode_names: Vec<&str> = self.modes.iter().map(|m| m.name.as_str()).collect();
        vec![
            self.id.clone(),
            self.name.clone(),
            self.key.clone(),
            mode_names.join(", "),
            self.variable_ids.len().to_string(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mode {
    #[serde(rename = "modeId")]
    pub mode_id: String,
    pub name: String,
}
