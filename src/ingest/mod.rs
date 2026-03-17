pub mod loader;
pub mod normalize;

use crate::schemas::layer1::Layer1Anchor;
use crate::schemas::policy::PolicyRule;
use anyhow::Result;
use std::path::Path;

pub fn load_and_normalize(tokens_path: &Path) -> Result<Vec<Layer1Anchor>> {
    let raw = loader::load_tokens(tokens_path)?;
    Ok(normalize::normalize_tokens(raw.anchors))
}

/// Load all policy rules from a directory tree.
///
/// Each `*.json` file under `policy_dir` must be at exactly
/// `<framework>/<category>/<code>.json` and pass identity validation.
pub fn load_policy(policy_dir: &Path) -> Result<Vec<PolicyRule>> {
    loader::load_policy_dir(policy_dir)
}
