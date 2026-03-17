use crate::policy::compiler::CompiledPolicyRule;
use crate::schemas::layer1::Layer1Anchor;
use crate::schemas::policy::{Condition, Op, Predicate};
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
    // Step 1: Language filter. "*" in the list means all languages are accepted.
    if !rule.languages.contains(&"*".to_string()) && !rule.languages.is_empty() {
        if !rule.languages.contains(&anchor.language) {
            return None;
        }
    }

    // Step 2: Scope filter (node_kinds, module_patterns, endpoint_protocols, require_tags).
    if !evaluate_scope(anchor, rule) {
        return None;
    }

    // Step 3: Recursive match condition tree.
    if !evaluate_condition(anchor, &rule.match_condition) {
        return None;
    }

    let reason = format!(
        "anchor '{}' matched rule '{}' via primary anchor '{}'",
        anchor.token, rule.id, rule.finding.primary_anchor
    );

    Some(RawMatch {
        match_id: Uuid::new_v4().to_string(),
        rule_id: rule.id.clone(),
        title: rule.title.clone(),
        category: rule.category.clone(),
        base_severity: rule.severity,
        file: anchor.file.clone(),
        line: anchor.line,
        column: anchor.column,
        matched_token: anchor.token.clone(),
        reason,
        anchor_id: anchor.id.clone(),
    })
}

// ---------------------------------------------------------------------------
// Scope evaluation
// ---------------------------------------------------------------------------

fn evaluate_scope(anchor: &Layer1Anchor, rule: &CompiledPolicyRule) -> bool {
    let scope = &rule.scope;

    // node_kinds: anchor's normalized_kind must be one of the listed kinds.
    if !scope.node_kinds.is_empty() && !scope.node_kinds.contains(&anchor.normalized_kind) {
        return false;
    }

    // module_patterns: file path must match at least one compiled glob.
    if !rule.path_matchers.is_empty() {
        let matched = rule.path_matchers.iter().any(|m| m.is_match(&anchor.file));
        if !matched {
            return false;
        }
    }

    // endpoint_protocols: check ast_metadata["protocol"].
    if !scope.endpoint_protocols.is_empty() {
        match anchor.ast_metadata.get("protocol").and_then(|v| v.as_str()) {
            Some(proto) => {
                if !scope.endpoint_protocols.iter().any(|p| p == proto) {
                    return false;
                }
            }
            None => return false,
        }
    }

    // require_tags: ast_metadata["semantic_tags"] must contain all required tags.
    if !scope.require_tags.is_empty() {
        let tags: Vec<&str> = match anchor.ast_metadata.get("semantic_tags") {
            Some(serde_json::Value::Array(arr)) => {
                arr.iter().filter_map(|v| v.as_str()).collect()
            }
            _ => return false,
        };
        for required in &scope.require_tags {
            if !tags.contains(&required.as_str()) {
                return false;
            }
        }
    }

    true
}

// ---------------------------------------------------------------------------
// Condition tree evaluation
// ---------------------------------------------------------------------------

fn evaluate_condition(anchor: &Layer1Anchor, condition: &Condition) -> bool {
    match condition {
        Condition::Predicate(pred) => evaluate_predicate(anchor, pred),
        Condition::All(all) => all.all.iter().all(|c| evaluate_condition(anchor, c)),
        Condition::Any(any) => any.any.iter().any(|c| evaluate_condition(anchor, c)),
        Condition::Not(not) => !evaluate_condition(anchor, &not.not),
    }
}

fn evaluate_predicate(anchor: &Layer1Anchor, pred: &Predicate) -> bool {
    let fact_val = resolve_fact(anchor, &pred.fact);

    match &pred.op {
        Op::Exists => fact_val.is_some(),

        Op::NotExists => fact_val.is_none(),

        Op::Equals => match (&fact_val, &pred.value) {
            (Some(fv), Some(pv)) => fv == pv,
            (None, None) => true,
            _ => false,
        },

        Op::NotEquals => match (&fact_val, &pred.value) {
            (Some(fv), Some(pv)) => fv != pv,
            _ => true,
        },

        Op::In => {
            if let (Some(fv), Some(serde_json::Value::Array(arr))) = (&fact_val, &pred.value) {
                arr.contains(fv)
            } else {
                false
            }
        }

        Op::NotIn => {
            if let (Some(fv), Some(serde_json::Value::Array(arr))) = (&fact_val, &pred.value) {
                !arr.contains(fv)
            } else {
                true
            }
        }

        Op::Contains => match (&fact_val, &pred.value) {
            (Some(serde_json::Value::String(s)), Some(serde_json::Value::String(needle))) => {
                s.contains(needle.as_str())
            }
            (Some(serde_json::Value::Array(arr)), Some(pv)) => arr.contains(pv),
            _ => false,
        },

        Op::NotContains => match (&fact_val, &pred.value) {
            (Some(serde_json::Value::String(s)), Some(serde_json::Value::String(needle))) => {
                !s.contains(needle.as_str())
            }
            (Some(serde_json::Value::Array(arr)), Some(pv)) => !arr.contains(pv),
            _ => true,
        },

        Op::Matches => {
            if let (
                Some(serde_json::Value::String(s)),
                Some(serde_json::Value::String(pattern)),
            ) = (&fact_val, &pred.value)
            {
                regex::Regex::new(pattern)
                    .map(|re| re.is_match(s))
                    .unwrap_or(false)
            } else {
                false
            }
        }

        // Taint analysis is not implemented; always returns false.
        Op::TaintReaches => false,
    }
}

// ---------------------------------------------------------------------------
// Fact resolution
// ---------------------------------------------------------------------------

/// Resolve a dotted fact path to a JSON value from the anchor.
///
/// Supported direct fields:
///   token, language, file, token_kind, normalized_kind,
///   layer1_rule_id, layer1_confidence
///
/// Nested facts:
///   `ast_metadata.<key>` — looks up `<key>` in `anchor.ast_metadata`.
///   Any other dotted path is tried as a plain key in `ast_metadata`.
fn resolve_fact(anchor: &Layer1Anchor, fact: &str) -> Option<serde_json::Value> {
    match fact {
        "token" => Some(serde_json::Value::String(anchor.token.clone())),
        "language" => Some(serde_json::Value::String(anchor.language.clone())),
        "file" => Some(serde_json::Value::String(anchor.file.clone())),
        "token_kind" => Some(serde_json::Value::String(anchor.token_kind.clone())),
        "normalized_kind" => Some(serde_json::Value::String(anchor.normalized_kind.clone())),
        "layer1_rule_id" => anchor
            .layer1_rule_id
            .as_ref()
            .map(|s| serde_json::Value::String(s.clone())),
        "layer1_confidence" => anchor
            .layer1_confidence
            .map(|f| serde_json::json!(f)),
        other => {
            let key = other
                .strip_prefix("ast_metadata.")
                .unwrap_or(other);
            anchor.ast_metadata.get(key).cloned()
        }
    }
}
