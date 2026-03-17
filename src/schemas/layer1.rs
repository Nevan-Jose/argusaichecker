use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer1Token {
    pub file: String,
    pub language: String,
    pub line: u32,
    pub column: u32,
    pub token: String,
    pub token_kind: String,
    pub normalized_kind: String,
    #[serde(default)]
    pub ast_metadata: HashMap<String, serde_json::Value>,
    pub layer1_rule_id: Option<String>,
    pub layer1_confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensFile {
    pub anchors: Vec<Layer1Token>,
}

#[derive(Debug, Clone)]
pub struct Layer1Anchor {
    pub id: String,
    pub file: String,
    pub language: String,
    pub line: u32,
    pub column: u32,
    pub token: String,
    pub token_kind: String,
    pub normalized_kind: String,
    pub ast_metadata: HashMap<String, serde_json::Value>,
    pub layer1_rule_id: Option<String>,
    pub layer1_confidence: Option<f64>,
}
