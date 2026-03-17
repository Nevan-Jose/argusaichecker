// Kept for backward compatibility with any direct callers.
// The main pipeline uses `provider::MockProvider` via the orchestrator.

use crate::schemas::review::{ReviewCandidate, TriageResult};

#[allow(dead_code)]
pub fn mock_triage(candidate: &ReviewCandidate) -> TriageResult {
    let needs_more_context = candidate.cluster_id.ends_with('0');
    let confirmed = !needs_more_context;

    TriageResult {
        cluster_id: candidate.cluster_id.clone(),
        confirmed,
        confidence: if confirmed { 0.82 } else { 0.31 },
        severity_adjustment: None,
        false_positive_reason: if !confirmed {
            Some("Mock helper flagged this as low-confidence and context-dependent.".to_string())
        } else {
            None
        },
        hidden_adjacent_faults: vec![],
        explanation: "[Mock review] Finding assessed by compatibility helper path.".to_string(),
        remediation: "Review and remediate according to policy guidelines.".to_string(),
        needs_more_context,
        requested_context: if needs_more_context {
            Some("Share context from adjacent lines for a final judgment.".to_string())
        } else {
            None
        },
        is_mock: true,
    }
}
