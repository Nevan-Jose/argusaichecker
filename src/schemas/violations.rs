use serde::{Deserialize, Serialize};
use crate::schemas::policy::Severity;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewStatus {
    Unreviewed,
    Reviewed,
    Escalated,
    ReviewFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMatch {
    pub match_id: String,
    pub rule_id: String,
    pub title: String,
    /// Free-form category string derived from the rule's path segment.
    pub category: String,
    pub base_severity: Severity,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub matched_token: String,
    pub reason: String,
    pub anchor_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationCluster {
    pub cluster_id: String,
    pub rule_id: String,
    pub title: String,
    /// Free-form category string derived from the rule's path segment.
    pub category: String,
    pub severity: Severity,
    pub file: String,
    pub start_line: u32,
    pub end_line: u32,
    pub matches: Vec<RawMatch>,
    pub match_count: usize,
    pub review_required: bool,
    pub policy_hints: Vec<String>,
    pub adjacent_bug_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeveritySummary {
    pub raw_match_count: usize,
    pub cluster_count: usize,
    pub by_severity: HashMap<String, usize>,
    pub by_category: HashMap<String, usize>,
    pub by_file: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationsReport {
    pub run_id: String,
    pub timestamp: String,
    pub summary: SeveritySummary,
    pub findings: Vec<ViolationCluster>,
}
