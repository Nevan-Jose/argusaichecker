use serde::{Deserialize, Serialize};
use crate::schemas::violations::{ViolationCluster, SeveritySummary};
use crate::schemas::review::TriageResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub reviewed_count: usize,
    pub confirmed_count: usize,
    pub false_positive_count: usize,
    pub escalation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalReport {
    pub run_id: String,
    pub timestamp: String,
    pub tokens_path: String,
    pub policy_path: String,
    pub source_dir: String,
    pub deterministic_summary: SeveritySummary,
    pub review_summary: ReviewSummary,
    pub findings: Vec<ViolationCluster>,
    pub triage_results: Vec<TriageResult>,
}
