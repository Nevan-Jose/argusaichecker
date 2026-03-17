use crate::schemas::policy::Severity;
use crate::schemas::review::{ReviewCandidate, ReviewMode};
use crate::schemas::violations::ViolationCluster;

pub fn rank_clusters(clusters: &[ViolationCluster]) -> Vec<ReviewCandidate> {
    let mut candidates: Vec<ReviewCandidate> = clusters
        .iter()
        .filter(|c| c.review_required)
        .map(|c| {
            let score = severity_score(c.severity) as f64 + (c.match_count as f64 * 0.1);
            ReviewCandidate {
                cluster_id: c.cluster_id.clone(),
                rank_score: score,
                reason_selected: format!(
                    "severity={}, match_count={}, review_required=true",
                    c.severity, c.match_count
                ),
                review_mode: ReviewMode::FirstPass,
            }
        })
        .collect();

    candidates.sort_by(|a, b| b.rank_score.partial_cmp(&a.rank_score).unwrap());
    candidates
}

fn severity_score(s: Severity) -> u32 {
    match s {
        Severity::Critical => 5,
        Severity::High => 4,
        Severity::Medium => 3,
        Severity::Low => 2,
        Severity::Info => 1,
    }
}
