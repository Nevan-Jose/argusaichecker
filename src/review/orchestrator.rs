use crate::context::packer;
use crate::review::provider::ReviewProvider;
use crate::schemas::review::{ReviewCandidate, TriageResult};
use crate::schemas::violations::ViolationCluster;
use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info, warn};

/// Run review for all selected candidates.
///
/// For each candidate:
///   1. Locate the corresponding cluster.
///   2. Pack a `ContextPacket` (loads source snippet if available).
///   3. Call the provider.
///   4. Collect results.
///
/// Candidates with no matching cluster are skipped and logged.
pub fn run(
    candidates: &[ReviewCandidate],
    clusters: &[ViolationCluster],
    source_dir: &Path,
    provider: &dyn ReviewProvider,
) -> Vec<TriageResult> {
    let cluster_map: HashMap<&str, &ViolationCluster> =
        clusters.iter().map(|c| (c.cluster_id.as_str(), c)).collect();

    let mut results = Vec::new();

    for candidate in candidates {
        let Some(cluster) = cluster_map.get(candidate.cluster_id.as_str()) else {
            warn!(
                "No cluster found for candidate {}, skipping",
                candidate.cluster_id
            );
            continue;
        };

        info!(
            "Reviewing finding {} (rule={}, severity={})",
            cluster.cluster_id, cluster.rule_id, cluster.severity
        );

        let packet = packer::pack(cluster, source_dir);
        let result = provider.review(&packet);

        if result.explanation.starts_with("Review failed") {
            error!(
                "Review failed for finding {} (rule={}): {}",
                cluster.cluster_id,
                cluster.rule_id,
                result.explanation
            );
        }

        if result.confirmed {
            info!(
                "Review confirmed finding {} with {:.0}% confidence",
                result.cluster_id,
                result.confidence * 100.0
            );
        } else {
            warn!(
                "Review rejected finding {} with {:.0}% confidence",
                result.cluster_id,
                result.confidence * 100.0
            );
        }

        if result.needs_more_context {
            warn!(
                "Review requires additional context for {}{}",
                result.cluster_id,
                result
                    .requested_context
                    .as_ref()
                    .map_or(String::new(), |context| format!(": {}", context))
            );
        }

        if let Some(fp_reason) = &result.false_positive_reason {
            warn!("Possible false positive reason for {}: {}", result.cluster_id, fp_reason);
        }

        results.push(result);
    }

    info!(
        "Review complete: {}/{} candidates reviewed",
        results.len(),
        candidates.len()
    );

    results
}
