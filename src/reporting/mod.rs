pub mod json_report;
pub mod markdown;
pub mod gitlab;

use crate::schemas::violations::{SeveritySummary, ViolationCluster};
use crate::schemas::review::TriageResult;
use anyhow::Result;
use std::path::Path;

pub fn write_violations(
    path: &Path,
    summary: &SeveritySummary,
    clusters: &[ViolationCluster],
) -> Result<()> {
    json_report::write_violations(path, summary, clusters)
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
    json_report::write_final_report(path, summary, clusters, reviews, tokens_path, policy_path, source_dir)
}

pub fn write_markdown(
    path: &Path,
    summary: &SeveritySummary,
    clusters: &[ViolationCluster],
    reviews: &[TriageResult],
) -> Result<()> {
    markdown::write(path, summary, clusters, reviews)
}
