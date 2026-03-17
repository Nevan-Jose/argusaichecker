use crate::schemas::policy::Severity;
use crate::schemas::review::TriageResult;

/// Parse and validate a raw JSON string returned by the AI provider.
///
/// Extracts the JSON block from the response (in case the model wraps it in
/// prose or markdown fences), then attempts to deserialize into a `TriageResult`.
/// Returns a detailed error string on failure.
pub fn parse_response(cluster_id: &str, raw: &str) -> Result<TriageResult, String> {
    let json_str = extract_json(raw);

    let v: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}. Raw: {json_str}"))?;

    let confirmed = v["confirmed"]
        .as_bool()
        .ok_or("missing or invalid 'confirmed'")?;
    let confidence = v["confidence"]
        .as_f64()
        .ok_or("missing or invalid 'confidence'")?
        .clamp(0.0, 1.0);

    let severity_adjustment = match v["severity_adjustment"].as_str() {
        Some("low")      => Some(Severity::Low),
        Some("medium")   => Some(Severity::Medium),
        Some("high")     => Some(Severity::High),
        Some("critical") => Some(Severity::Critical),
        // "info" is no longer a valid severity in the new schema; treat as no adjustment.
        None | Some("null") | Some("") | Some("info") => None,
        Some(other) => {
            return Err(format!("invalid severity_adjustment value: '{other}'"));
        }
    };

    let false_positive_reason = v["false_positive_reason"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let hidden_adjacent_faults = v["hidden_adjacent_faults"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let explanation = v["explanation"]
        .as_str()
        .ok_or("missing 'explanation'")?
        .to_string();

    let remediation = v["remediation"]
        .as_str()
        .ok_or("missing 'remediation'")?
        .to_string();

    let needs_more_context = v["needs_more_context"].as_bool().unwrap_or(false);

    let requested_context = v["requested_context"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    Ok(TriageResult {
        cluster_id: cluster_id.to_string(),
        confirmed,
        confidence,
        severity_adjustment,
        false_positive_reason,
        hidden_adjacent_faults,
        explanation,
        remediation,
        needs_more_context,
        requested_context,
        is_mock: false,
    })
}

/// Extract the first JSON object `{...}` found in `text`.
/// Falls back to the full trimmed text if no braces are found.
fn extract_json(text: &str) -> &str {
    // Strip markdown code fences if present
    let text = text.trim();
    let text = text
        .strip_prefix("```json")
        .or_else(|| text.strip_prefix("```"))
        .unwrap_or(text);
    let text = text.strip_suffix("```").unwrap_or(text).trim();

    // Find the outermost { ... }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end >= start {
                return &text[start..=end];
            }
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_JSON: &str = r#"{
  "confirmed": true,
  "confidence": 0.9,
  "severity_adjustment": null,
  "false_positive_reason": null,
  "hidden_adjacent_faults": ["Possible insecure logging nearby"],
  "explanation": "The secret is hardcoded.",
  "remediation": "Move to a secrets manager.",
  "needs_more_context": false,
  "requested_context": null
}"#;

    #[test]
    fn test_valid_response_parses() {
        let result = parse_response("c1", VALID_JSON);
        assert!(result.is_ok(), "{:?}", result.err());
        let r = result.unwrap();
        assert!(r.confirmed);
        assert!((r.confidence - 0.9).abs() < 0.01);
        assert_eq!(r.hidden_adjacent_faults.len(), 1);
        assert!(!r.is_mock);
    }

    #[test]
    fn test_markdown_fenced_json_parses() {
        let wrapped = format!("```json\n{}\n```", VALID_JSON);
        let result = parse_response("c1", &wrapped);
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_invalid_json_returns_error() {
        let result = parse_response("c1", "not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_field_returns_error() {
        let bad = r#"{"confirmed": true}"#;
        let result = parse_response("c1", bad);
        assert!(result.is_err());
    }
}
