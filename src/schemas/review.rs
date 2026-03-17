use serde::{Deserialize, Serialize};
use crate::schemas::policy::Severity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewMode {
    None,
    FirstPass,
    EscalatedPass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCandidate {
    pub cluster_id: String,
    pub rank_score: f64,
    pub reason_selected: String,
    pub review_mode: ReviewMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageResult {
    pub cluster_id: String,
    pub confirmed: bool,
    pub confidence: f64,
    pub severity_adjustment: Option<Severity>,
    pub false_positive_reason: Option<String>,
    pub hidden_adjacent_faults: Vec<String>,
    pub explanation: String,
    pub remediation: String,
    pub needs_more_context: bool,
    pub requested_context: Option<String>,
    /// True when produced by the mock provider; false when a real model was called.
    #[serde(default)]
    pub is_mock: bool,
}
