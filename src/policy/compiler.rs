use crate::schemas::policy::{Condition, Finding, Mappings, PolicyRule, Scope, Severity};
use anyhow::{Context, Result};
use globset::{Glob, GlobMatcher};

/// A compiled policy rule ready for matching.
///
/// `scope.module_patterns` are pre-compiled into glob matchers.
/// All other fields are carried over from the parsed `PolicyRule`.
pub struct CompiledPolicyRule {
    /// Stable rule identity: `<framework>.<category>.<code>`.
    pub id: String,
    pub title: String,
    pub description: String,
    /// Category string from the rule's path segment (e.g. "injection").
    pub category: String,
    pub severity: Severity,
    /// Languages supported by this rule. Contains `"*"` to match all.
    pub languages: Vec<String>,
    pub scope: Scope,
    /// Pre-compiled glob matchers derived from `scope.module_patterns`.
    pub path_matchers: Vec<GlobMatcher>,
    /// Recursive match condition tree.
    pub match_condition: Condition,
    pub finding: Finding,
    pub references: Vec<String>,
    pub mappings: Option<Mappings>,
}

pub fn compile(rule: PolicyRule) -> Result<CompiledPolicyRule> {
    let path_matchers = rule
        .scope
        .module_patterns
        .iter()
        .map(|p| {
            Glob::new(p)
                .with_context(|| format!("Invalid glob in rule '{}': {}", rule.id, p))
                .map(|g| g.compile_matcher())
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(CompiledPolicyRule {
        category: rule.policy.category.clone(),
        id: rule.id,
        title: rule.title,
        description: rule.description,
        severity: rule.severity,
        languages: rule
            .languages
            .into_iter()
            .map(|l| l.to_lowercase())
            .collect(),
        scope: rule.scope,
        path_matchers,
        match_condition: rule.match_condition,
        finding: rule.finding,
        references: rule.references,
        mappings: rule.mappings,
    })
}
