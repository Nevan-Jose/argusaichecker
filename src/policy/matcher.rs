use crate::policy::compiler::CompiledPolicyRule;
use crate::schemas::layer1::Layer1Anchor;
use crate::schemas::violations::RawMatch;
use uuid::Uuid;

pub fn match_all(anchors: &[Layer1Anchor], rules: &[CompiledPolicyRule]) -> Vec<RawMatch> {
    let mut results = Vec::new();
    for anchor in anchors {
        for rule in rules {
            if let Some(m) = try_match(anchor, rule) {
                results.push(m);
            }
        }
    }
    results
}

fn try_match(anchor: &Layer1Anchor, rule: &CompiledPolicyRule) -> Option<RawMatch> {
    // Language filter
    if !rule.languages.is_empty() && !rule.languages.contains(&anchor.language) {
        return None;
    }

    // Path filter
    if !rule.path_matchers.is_empty() {
        let matched_path = rule.path_matchers.iter().any(|m| m.is_match(&anchor.file));
        if !matched_path {
            return None;
        }
    }

    // Token pattern match
    let matched_pattern = rule.token_patterns.iter().find(|re| re.is_match(&anchor.token))?;
    let reason = format!("token '{}' matched pattern '{}'", anchor.token, matched_pattern.as_str());

    Some(RawMatch {
        match_id: Uuid::new_v4().to_string(),
        rule_id: rule.rule_id.clone(),
        title: rule.title.clone(),
        category: rule.category,
        base_severity: rule.severity,
        file: anchor.file.clone(),
        line: anchor.line,
        column: anchor.column,
        matched_token: anchor.token.clone(),
        reason,
        anchor_id: anchor.id.clone(),
    })
}
