use crate::schemas::layer1::{Layer1Token, TokensFile};
use crate::schemas::policy::PolicyFile;
use anyhow::{Context, Result};
use std::path::Path;

/// Load and parse a tokens file from Layer 1.
///
/// Accepts two JSON shapes:
///   1. `{ "anchors": [ ... ] }` — current sample/spec format.
///   2. `[ ... ]`                — bare array, in case Layer 1 delivers it directly.
///
/// If neither parses successfully, both error messages are included to help
/// the teammate diagnose field-name mismatches or format changes quickly.
pub fn load_tokens(path: &Path) -> Result<TokensFile> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read tokens file: {:?}", path))?;

    // Try the wrapped format first (current spec: { "anchors": [...] }).
    let wrapped_err = match serde_json::from_str::<TokensFile>(&data) {
        Ok(f) => {
            validate_tokens(&f, path)?;
            return Ok(f);
        }
        Err(e) => e,
    };

    // Fall back to a bare array.
    match serde_json::from_str::<Vec<Layer1Token>>(&data) {
        Ok(anchors) => {
            let f = TokensFile { anchors };
            validate_tokens(&f, path)?;
            Ok(f)
        }
        Err(array_err) => anyhow::bail!(
            "Failed to parse tokens file {:?}.\n  \
             Tried {{\"anchors\":[...]}}: {}\n  \
             Tried bare array [...]: {}\n  \
             Check that your Layer 1 output uses the expected field names \
             (file, language, line, column, token, token_kind, normalized_kind).",
            path,
            wrapped_err,
            array_err
        ),
    }
}

/// Validate the loaded token file to surface field-name issues early.
fn validate_tokens(f: &TokensFile, path: &Path) -> Result<()> {
    if f.anchors.is_empty() {
        tracing::warn!("Tokens file {:?} contains no anchors — scan will produce no findings", path);
        return Ok(());
    }
    let first = &f.anchors[0];
    if first.file.is_empty() {
        anyhow::bail!(
            "First anchor in {:?} has an empty 'file' field. \
             Check whether your Layer 1 output uses 'file' or a different key (e.g. 'path', 'filepath').",
            path
        );
    }
    if first.token.is_empty() {
        anyhow::bail!(
            "First anchor in {:?} has an empty 'token' field. \
             Expected the matched token text to be present.",
            path
        );
    }
    Ok(())
}

pub fn load_policy_file(path: &Path) -> Result<PolicyFile> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read policy file: {:?}", path))?;
    serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse policy JSON: {:?}", path))
}
