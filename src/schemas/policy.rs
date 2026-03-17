use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

// ---------------------------------------------------------------------------
// Policy identity
// ---------------------------------------------------------------------------

/// The `policy` object embedded in every rule file.
/// Its fields must match the file path: `<framework>/<category>/<code>.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMeta {
    pub framework: String,
    pub category: String,
    pub code: String,
}

// ---------------------------------------------------------------------------
// Scope
// ---------------------------------------------------------------------------

/// Scope filters applied before the match condition tree.
/// An empty scope passes all anchors.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Scope {
    #[serde(default)]
    pub node_kinds: Vec<String>,
    #[serde(default)]
    pub module_patterns: Vec<String>,
    #[serde(default)]
    pub endpoint_protocols: Vec<String>,
    #[serde(default)]
    pub require_tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Match condition tree
// ---------------------------------------------------------------------------

/// Operator for a leaf predicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    Equals,
    NotEquals,
    In,
    NotIn,
    Contains,
    NotContains,
    Exists,
    NotExists,
    Matches,
    TaintReaches,
}

/// A leaf predicate: `{ "fact": "...", "op": "...", "value": ... }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predicate {
    pub fact: String,
    pub op: Op,
    pub value: Option<serde_json::Value>,
}

/// `{ "all": [ <condition>, ... ] }` — all children must be true.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllCondition {
    pub all: Vec<Condition>,
}

/// `{ "any": [ <condition>, ... ] }` — at least one child must be true.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnyCondition {
    pub any: Vec<Condition>,
}

/// `{ "not": <condition> }` — child must be false.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotCondition {
    pub not: Box<Condition>,
}

/// Recursive condition node. Deserialized from JSON using the untagged
/// strategy: serde tries each variant in order and picks the first that fits
/// the shape of the incoming object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    All(AllCondition),
    Any(AnyCondition),
    Not(NotCondition),
    Predicate(Predicate),
}

// ---------------------------------------------------------------------------
// Finding
// ---------------------------------------------------------------------------

/// Describes how a matched finding should be reported.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub message: String,
    /// Fact path that identifies the primary anchor for the finding (required).
    pub primary_anchor: String,
    #[serde(default)]
    pub related: Vec<String>,
}

// ---------------------------------------------------------------------------
// Optional metadata
// ---------------------------------------------------------------------------

/// Compliance framework mappings attached to a rule.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mappings {
    #[serde(default)]
    pub soc2: Vec<String>,
    #[serde(default)]
    pub gdpr: Vec<String>,
    #[serde(default)]
    pub owasp: Vec<String>,
}

// ---------------------------------------------------------------------------
// PolicyRule — the top-level object in each rule file
// ---------------------------------------------------------------------------

/// One rule, loaded from `<framework>/<category>/<code>.json`.
///
/// Identity invariants enforced by the loader:
/// - `id` == `policy.framework` + "." + `policy.category` + "." + `policy.code`
/// - path segments match `policy.framework`, `policy.category`, `policy.code`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Stable identifier: `<framework>.<category>.<code>`.
    pub id: String,
    pub title: String,
    pub description: String,
    pub policy: PolicyMeta,
    pub severity: Severity,
    /// Supported languages. `["*"]` means all languages.
    pub languages: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub scope: Scope,
    /// Recursive match condition tree.
    #[serde(rename = "match")]
    pub match_condition: Condition,
    pub finding: Finding,
    #[serde(default)]
    pub mappings: Option<Mappings>,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub defaults: Option<HashMap<String, serde_json::Value>>,
}
