use crate::schemas::policy::{Category, PolicyRule, Severity};
use anyhow::{Context, Result};
use globset::{Glob, GlobMatcher};
use regex::Regex;

pub struct CompiledPolicyRule {
    pub rule_id: String,
    pub title: String,
    pub description: String,
    pub category: Category,
    pub severity: Severity,
    pub languages: Vec<String>,
    pub token_patterns: Vec<Regex>,
    pub path_matchers: Vec<GlobMatcher>,
    pub dedupe_radius_lines: u32,
    pub review_required: bool,
    pub policy_hints: Vec<String>,
    pub adjacent_bug_checks: Vec<String>,
}

pub fn compile(rule: PolicyRule) -> Result<CompiledPolicyRule> {
    let token_patterns = rule
        .token_patterns
        .iter()
        .map(|p| Regex::new(p).with_context(|| format!("Invalid regex in rule {}: {}", rule.rule_id, p)))
        .collect::<Result<Vec<_>>>()?;

    let path_matchers = rule
        .path_patterns
        .iter()
        .map(|p| {
            Glob::new(p)
                .with_context(|| format!("Invalid glob in rule {}: {}", rule.rule_id, p))
                .map(|g| g.compile_matcher())
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(CompiledPolicyRule {
        rule_id: rule.rule_id,
        title: rule.title,
        description: rule.description,
        category: rule.category,
        severity: rule.severity,
        languages: rule.languages.into_iter().map(|l| l.to_lowercase()).collect(),
        token_patterns,
        path_matchers,
        dedupe_radius_lines: rule.dedupe_radius_lines,
        review_required: rule.review_required,
        policy_hints: rule.policy_hints,
        adjacent_bug_checks: rule.adjacent_bug_checks,
    })
}
