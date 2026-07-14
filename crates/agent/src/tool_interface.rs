use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A tool definition in JSON-Schema / MCP-compatible format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input: ToolSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<ToolSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side_effects: Option<Vec<String>>,
}

/// JSON Schema for tool input/output.
/// Minimal representation — only what's needed for agent reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, ToolProperty>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A property within a tool schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProperty {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enum_: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ToolSchema>>,
}

/// A complete tool interface file (`./ai/tools.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInterfaceFile {
    pub tools: Vec<ToolDefinition>,
}

impl ToolInterfaceFile {
    pub fn from_json_str(json: &str) -> Result<Self, ToolInterfaceError> {
        serde_json::from_str(json).map_err(|e| ToolInterfaceError::Parse(e.to_string()))
    }

    pub fn to_json_string(&self) -> Result<String, ToolInterfaceError> {
        serde_json::to_string_pretty(self).map_err(|e| ToolInterfaceError::Serialize(e.to_string()))
    }

    pub fn from_json_value(value: serde_json::Value) -> Result<Self, ToolInterfaceError> {
        serde_json::from_value(value).map_err(|e| ToolInterfaceError::Parse(e.to_string()))
    }
}

/// Fingerprint of a tool interface for structural similarity matching.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToolFingerprint {
    /// Sorted tool names joined by delimiter
    pub tool_names_hash: String,
    /// Sorted parameter signatures
    pub param_signatures: Vec<String>,
    /// Number of tools
    pub tool_count: usize,
}

impl ToolFingerprint {
    pub fn from_interface(tools: &[ToolDefinition]) -> Self {
        let mut names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        names.sort();

        let mut tool_names_hash = String::new();
        for name in &names {
            if !tool_names_hash.is_empty() {
                tool_names_hash.push(',');
            }
            tool_names_hash.push_str(name);
        }

        let mut param_signatures: Vec<String> = tools
            .iter()
            .map(|t| {
                let params = t
                    .input
                    .properties
                    .as_ref()
                    .map(|props| {
                        let mut keys: Vec<&String> = props.keys().collect();
                        keys.sort();
                        keys.iter()
                            .map(|k| {
                                format!(
                                    "{}:{}",
                                    k,
                                    props.get(*k).map(|p| p.type_.as_str()).unwrap_or("unknown")
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .unwrap_or_default();
                format!("{}:({})", t.name, params)
            })
            .collect();
        param_signatures.sort();

        Self { tool_names_hash, param_signatures, tool_count: tools.len() }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ToolInterfaceError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("serialize error: {0}")]
    Serialize(String),
}
