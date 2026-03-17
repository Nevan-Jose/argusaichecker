use crate::schemas::layer1::{Layer1Token, TokensFile};
use crate::schemas::policy::PolicyRule;
use anyhow::{Context, Result};
use std::path::{Path, Component};
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Token loading
// ---------------------------------------------------------------------------

/// Load and parse a tokens file from Layer 1.
///
/// Accepts two JSON shapes:
///   1. `{ "anchors": [ ... ] }` — current sample/spec format.
///   2. `[ ... ]`                — bare array, in case Layer 1 delivers it directly.
pub fn load_tokens(path: &Path) -> Result<TokensFile> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read tokens file: {:?}", path))?;

    let wrapped_err = match serde_json::from_str::<TokensFile>(&data) {
        Ok(f) => {
            validate_tokens(&f, path)?;
            return Ok(f);
        }
        Err(e) => e,
    };

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

// ---------------------------------------------------------------------------
// Policy directory loading
// ---------------------------------------------------------------------------

/// Recursively load all `*.json` rule files from `dir`.
///
/// Every file is validated against:
/// - The expected path format: `<framework>/<category>/<code>.json`
/// - Identity consistency: `id` == `policy.framework.category.code`
///   and each path segment matches the corresponding `policy.*` field.
///
/// Returns an error if any file fails validation, ensuring the entire rule
/// set is coherent before the pipeline starts.
pub fn load_policy_dir(dir: &Path) -> Result<Vec<PolicyRule>> {
    if !dir.exists() {
        anyhow::bail!("Policy directory {:?} does not exist", dir);
    }
    if !dir.is_dir() {
        anyhow::bail!("Policy path {:?} is not a directory — pass a directory, not a single file", dir);
    }

    let mut rules = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                == Some("json")
        })
    {
        let path = entry.path();
        let rule = load_and_validate_rule(path, dir)
            .with_context(|| format!("Loading rule file {:?}", path))?;
        rules.push(rule);
    }

    if rules.is_empty() {
        tracing::warn!("No rule files found in policy directory {:?}", dir);
    } else {
        tracing::info!("Loaded {} policy rules from {:?}", rules.len(), dir);
    }

    Ok(rules)
}

fn load_and_validate_rule(path: &Path, base_dir: &Path) -> Result<PolicyRule> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read rule file: {:?}", path))?;

    let rule: PolicyRule = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse rule JSON at {:?}", path))?;

    let rel_path = path
        .strip_prefix(base_dir)
        .with_context(|| {
            format!("Path {:?} is not under base dir {:?}", path, base_dir)
        })?;

    validate_rule_identity(&rule, rel_path, path)?;

    Ok(rule)
}

/// Enforce path-to-identity consistency.
///
/// The file must be at exactly three path components deep:
///   `<framework>/<category>/<code>.json`
/// and the rule's `id`, `policy.framework`, `policy.category`, and
/// `policy.code` must all agree with those path segments.
fn validate_rule_identity(rule: &PolicyRule, rel_path: &Path, full_path: &Path) -> Result<()> {
    let components: Vec<&str> = rel_path
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();

    if components.len() != 3 {
        anyhow::bail!(
            "Rule file {:?} must be at <framework>/<category>/<code>.json \
             (found {} path components relative to policy directory, expected 3)",
            full_path,
            components.len()
        );
    }

    let framework = components[0];
    let category = components[1];
    let code_file = components[2];

    let code = code_file.strip_suffix(".json").ok_or_else(|| {
        anyhow::anyhow!(
            "Rule file {:?} does not end with .json",
            full_path
        )
    })?;

    let expected_id = format!("{}.{}.{}", framework, category, code);

    if rule.id != expected_id {
        anyhow::bail!(
            "Rule id mismatch in {:?}: field 'id' is '{}' but path implies '{}'",
            full_path,
            rule.id,
            expected_id
        );
    }

    if rule.policy.framework != framework {
        anyhow::bail!(
            "Rule {:?}: policy.framework '{}' does not match path segment '{}'",
            full_path,
            rule.policy.framework,
            framework
        );
    }

    if rule.policy.category != category {
        anyhow::bail!(
            "Rule {:?}: policy.category '{}' does not match path segment '{}'",
            full_path,
            rule.policy.category,
            category
        );
    }

    if rule.policy.code != code {
        anyhow::bail!(
            "Rule {:?}: policy.code '{}' does not match path segment '{}'",
            full_path,
            rule.policy.code,
            code
        );
    }

    // Require a non-empty primary anchor
    if rule.finding.primary_anchor.is_empty() {
        anyhow::bail!(
            "Rule {:?}: finding.primary_anchor must not be empty",
            full_path
        );
    }

    Ok(())
}
