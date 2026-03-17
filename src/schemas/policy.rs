use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Secrets,
    Crypto,
    Injection,
    Execution,
    Auth,
    Compliance,
    Deserialization,
    Other,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Secrets => write!(f, "secrets"),
            Category::Crypto => write!(f, "crypto"),
            Category::Injection => write!(f, "injection"),
            Category::Execution => write!(f, "execution"),
            Category::Auth => write!(f, "auth"),
            Category::Compliance => write!(f, "compliance"),
            Category::Deserialization => write!(f, "deserialization"),
            Category::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub rule_id: String,
    pub title: String,
    pub description: String,
    pub category: Category,
    pub severity: Severity,
    pub languages: Vec<String>,
    #[serde(default)]
    pub path_patterns: Vec<String>,
    pub token_patterns: Vec<String>,
    #[serde(default)]
    pub metadata_filters: serde_json::Value,
    pub dedupe_radius_lines: u32,
    pub review_required: bool,
    #[serde(default)]
    pub policy_hints: Vec<String>,
    #[serde(default)]
    pub adjacent_bug_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyFile {
    pub rules: Vec<PolicyRule>,
}
