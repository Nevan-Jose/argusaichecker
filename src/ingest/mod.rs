pub mod loader;
pub mod normalize;

use crate::schemas::layer1::Layer1Anchor;
use anyhow::Result;
use std::path::Path;

pub fn load_and_normalize(tokens_path: &Path) -> Result<Vec<Layer1Anchor>> {
    let raw = loader::load_tokens(tokens_path)?;
    Ok(normalize::normalize_tokens(raw.anchors))
}

pub fn load_policy(policy_path: &Path) -> Result<Vec<crate::schemas::policy::PolicyRule>> {
    let pf = loader::load_policy_file(policy_path)?;
    Ok(pf.rules)
}
