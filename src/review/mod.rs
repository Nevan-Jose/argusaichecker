pub mod cache;
pub mod mock;
pub mod orchestrator;
pub mod prompt;
pub mod provider;
pub mod validator;

use crate::review::provider::{GeminiProvider, MockProvider, ReviewProvider};
use crate::schemas::review::{ReviewCandidate, TriageResult};
use crate::schemas::violations::ViolationCluster;
use std::path::Path;
use tracing::{info, warn};

/// Run the review pipeline, choosing the provider based on `use_mock` and
/// environment configuration.
///
/// - If `use_mock` is true → offline `MockProvider`.
/// - Else if `GEMINI_API_KEY` is set → `GeminiProvider`.
/// - Else → `MockProvider` with a warning.
pub fn run_review(
    use_mock: bool,
    candidates: &[ReviewCandidate],
    clusters: &[ViolationCluster],
    source_dir: &Path,
) -> Vec<TriageResult> {
    if candidates.is_empty() {
        return vec![];
    }

    let provider: Box<dyn ReviewProvider> = if use_mock {
        info!("Using mock review provider (--mock-review flag set)");
        Box::new(MockProvider)
    } else if let Some(p) = GeminiProvider::from_env() {
        info!("Using Gemini live review provider");
        Box::new(p)
    } else {
        warn!(
            "GEMINI_API_KEY not set — falling back to mock provider. \
             Set GEMINI_API_KEY to enable live Gemini review."
        );
        Box::new(MockProvider)
    };

    orchestrator::run(candidates, clusters, source_dir, provider.as_ref())
}

/// Backward-compatible helper for tests that do not pass source context.
#[allow(dead_code)]
pub fn run_mock_review(candidates: &[ReviewCandidate]) -> Vec<TriageResult> {
    candidates.iter().map(|c| mock::mock_triage(c)).collect()
}
