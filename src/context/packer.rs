use crate::context::snippet;
use crate::context::source_loader;
use crate::schemas::violations::ViolationCluster;
use tracing::warn;
use std::path::Path;

/// A compact, self-contained packet of information sent to the AI review layer.
///
/// Contains only what is needed for a single finding review — no full files,
/// no cross-file context on the first pass.
#[derive(Debug, Clone)]
pub struct ContextPacket {
    pub cluster_id: String,
    pub rule_id: String,
    pub title: String,
    pub category: String,
    pub severity: String,
    pub file: String,
    pub language: String,
    pub start_line: u32,
    pub end_line: u32,
    /// The specific token text that triggered the match.
    pub matched_token: String,
    /// Human-readable explanation of why the pattern matched.
    pub match_reason: String,
    /// Numbered code snippet centred on the finding (empty if source unavailable).
    pub snippet: String,
    pub policy_hints: Vec<String>,
    pub adjacent_bug_checks: Vec<String>,
}

/// Lines above and below the finding to include in the snippet.
const SNIPPET_RADIUS: u32 = 6;

/// Build a `ContextPacket` for a single cluster.
///
/// Attempts to load the source file from disk. If the file cannot be found
/// the packet is still returned with an empty snippet so the review can
/// proceed on deterministic evidence alone.
pub fn pack(cluster: &ViolationCluster, source_dir: &Path) -> ContextPacket {
    let first_match = cluster.matches.first();
    let matched_token = first_match
        .map(|m| m.matched_token.clone())
        .unwrap_or_default();
    let match_reason = first_match
        .map(|m| m.reason.clone())
        .unwrap_or_default();

    let language = source_loader::language_from_path(&cluster.file).to_string();

    let snippet = match source_loader::load_file(&cluster.file, source_dir) {
        Ok((content, _)) => snippet::extract_numbered(&content, cluster.start_line, SNIPPET_RADIUS),
        Err(err) => {
            warn!(
                "Could not load source snippet for cluster {} from '{}': {}",
                cluster.cluster_id,
                cluster.file,
                err
            );
            String::new()
        }
    };

    ContextPacket {
        cluster_id: cluster.cluster_id.clone(),
        rule_id: cluster.rule_id.clone(),
        title: cluster.title.clone(),
        category: cluster.category.to_string(),
        severity: cluster.severity.to_string(),
        file: cluster.file.clone(),
        language,
        start_line: cluster.start_line,
        end_line: cluster.end_line,
        matched_token,
        match_reason,
        snippet,
        policy_hints: cluster.policy_hints.clone(),
        adjacent_bug_checks: cluster.adjacent_bug_checks.clone(),
    }
}
