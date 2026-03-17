use crate::schemas::report::{FinalReport, ReviewSummary};
use crate::schemas::review::TriageResult;
use crate::schemas::violations::{SeveritySummary, ViolationCluster, ViolationsReport};
use anyhow::Result;
use chrono::Utc;
use std::path::Path;
use uuid::Uuid;

pub fn write_violations(
    path: &Path,
    summary: &SeveritySummary,
    clusters: &[ViolationCluster],
) -> Result<()> {
    let report = ViolationsReport {
        run_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        summary: summary.clone(),
        findings: clusters.to_vec(),
    };
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn write_final_report(
    path: &Path,
    summary: &SeveritySummary,
    clusters: &[ViolationCluster],
    reviews: &[TriageResult],
    tokens_path: &str,
    policy_path: &str,
    source_dir: &str,
) -> Result<()> {
    let confirmed_count = reviews.iter().filter(|r| r.confirmed).count();
    let false_positive_count = reviews.iter().filter(|r| !r.confirmed).count();
    let escalation_count = reviews.iter().filter(|r| r.needs_more_context).count();

    let review_summary = ReviewSummary {
        reviewed_count: reviews.len(),
        confirmed_count,
        false_positive_count,
        escalation_count,
    };

    let report = FinalReport {
        run_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        tokens_path: tokens_path.to_string(),
        policy_path: policy_path.to_string(),
        source_dir: source_dir.to_string(),
        deterministic_summary: summary.clone(),
        review_summary,
        findings: clusters.to_vec(),
        triage_results: reviews.to_vec(),
    };
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(path, json)?;
    Ok(())
}
