pub mod compiler;
pub mod matcher;
pub mod dedupe;
pub mod severity;

use crate::schemas::layer1::Layer1Anchor;
use crate::schemas::policy::PolicyRule;
use crate::schemas::violations::{RawMatch, SeveritySummary, ViolationCluster};
use anyhow::Result;

pub use compiler::CompiledPolicyRule;

pub fn compile_rules(rules: Vec<PolicyRule>) -> Result<Vec<CompiledPolicyRule>> {
    rules.into_iter().map(compiler::compile).collect()
}

pub fn match_anchors(anchors: &[Layer1Anchor], rules: &[CompiledPolicyRule]) -> Vec<RawMatch> {
    matcher::match_all(anchors, rules)
}

pub fn dedupe_and_cluster(matches: Vec<RawMatch>, rules: &[CompiledPolicyRule]) -> Vec<ViolationCluster> {
    dedupe::cluster_with_rules(matches, rules)
}

pub fn build_summary(clusters: &[ViolationCluster]) -> SeveritySummary {
    severity::summarize(clusters)
}
