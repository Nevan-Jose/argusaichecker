use crate::policy::compiler::CompiledPolicyRule;
use crate::schemas::policy::Severity;
use crate::schemas::violations::{RawMatch, ViolationCluster};
use std::collections::HashMap;
use uuid::Uuid;

pub fn cluster_with_rules(
    mut matches: Vec<RawMatch>,
    rules: &[CompiledPolicyRule],
) -> Vec<ViolationCluster> {
    let rule_map: HashMap<&str, &CompiledPolicyRule> =
        rules.iter().map(|r| (r.id.as_str(), r)).collect();

    // Group by (file, rule_id)
    let mut groups: HashMap<(String, String), Vec<RawMatch>> = HashMap::new();
    for m in matches.drain(..) {
        groups
            .entry((m.file.clone(), m.rule_id.clone()))
            .or_default()
            .push(m);
    }

    let mut clusters = Vec::new();
    for ((file, rule_id), mut group) in groups {
        group.sort_by_key(|m| m.line);

        let rule = rule_map.get(rule_id.as_str()).copied();

        // Deduplicate nearby hits within a fixed radius.
        let radius: u32 = 5;
        let sub_clusters = merge_nearby(group, radius);

        for sub in sub_clusters {
            let first = &sub[0];
            let title = first.title.clone();
            let category = first.category.clone();
            let severity = first.base_severity;
            let start_line = sub.first().map(|m| m.line).unwrap_or(0);
            let end_line = sub.last().map(|m| m.line).unwrap_or(0);
            let count = sub.len();

            // Derive review_required from severity: high and critical always
            // require a review pass.
            let review_required = rule.map(|r| {
                matches!(r.severity, Severity::High | Severity::Critical)
            }).unwrap_or(false);

            // Use rule references as policy hints for the reporting layer.
            let policy_hints = rule
                .map(|r| r.references.clone())
                .unwrap_or_default();

            clusters.push(ViolationCluster {
                cluster_id: Uuid::new_v4().to_string(),
                rule_id: rule_id.clone(),
                title,
                category,
                severity,
                file: file.clone(),
                start_line,
                end_line,
                matches: sub,
                match_count: count,
                review_required,
                policy_hints,
                adjacent_bug_checks: Vec::new(),
            });
        }
    }

    clusters.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then(a.file.cmp(&b.file))
            .then(a.start_line.cmp(&b.start_line))
    });
    clusters
}

fn merge_nearby(sorted: Vec<RawMatch>, radius: u32) -> Vec<Vec<RawMatch>> {
    if sorted.is_empty() {
        return vec![];
    }
    let mut result: Vec<Vec<RawMatch>> = Vec::new();
    let mut current: Vec<RawMatch> = Vec::new();

    for m in sorted {
        if current.is_empty() {
            current.push(m);
        } else {
            let last_line = current.last().unwrap().line;
            if m.line.saturating_sub(last_line) <= radius {
                current.push(m);
            } else {
                result.push(current);
                current = vec![m];
            }
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}
