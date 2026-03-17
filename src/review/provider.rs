use crate::context::ContextPacket;
use crate::review::{prompt, validator};
use crate::schemas::review::TriageResult;
use tracing::{info, warn};

/// Core abstraction over any review backend.
pub trait ReviewProvider {
    fn review(&self, packet: &ContextPacket) -> TriageResult;
}

// ── Mock provider ─────────────────────────────────────────────────────────────

/// Offline mock that returns a plausible-looking triage result without any
/// network call. Used in `--mock-review` mode and for offline testing.
pub struct MockProvider;

impl ReviewProvider for MockProvider {
    fn review(&self, packet: &ContextPacket) -> TriageResult {
        let plan = mock_review_plan(packet);
        TriageResult {
            cluster_id: packet.cluster_id.clone(),
            confirmed: plan.confirmed,
            confidence: plan.confidence,
            severity_adjustment: None,
            false_positive_reason: plan.false_positive_reason.clone(),
            hidden_adjacent_faults: mock_adjacent_faults(packet),
            explanation: mock_explanation(packet, &plan),
            remediation: mock_remediation(packet),
            needs_more_context: plan.needs_more_context,
            requested_context: plan.requested_context,
            is_mock: true,
        }
    }
}

#[derive(Default)]
struct MockReviewPlan {
    confirmed: bool,
    confidence: f64,
    false_positive_reason: Option<String>,
    needs_more_context: bool,
    requested_context: Option<String>,
}

fn mock_review_plan(packet: &ContextPacket) -> MockReviewPlan {
    let token = packet.matched_token.to_lowercase();

    if packet.rule_id == "POL-SQL-001" && (token.contains("select") || token.contains("insert") || token.contains("update")) {
        return MockReviewPlan {
            confirmed: false,
            confidence: 0.34,
            false_positive_reason: Some(
                "Query composition should be verified against prepared statements before escalation."
                    .to_string(),
            ),
            needs_more_context: true,
            requested_context: Some(
                "Share caller input variables and whether database APIs are parameterized."
                    .to_string(),
            ),
        };
    }

    if packet.rule_id == "POL-SECRET-001" && token.contains("print") {
        return MockReviewPlan {
            confirmed: false,
            confidence: 0.28,
            false_positive_reason: Some(
                "This looks like debug/output context; confirm this literal is not a real secret."
                    .to_string(),
            ),
            needs_more_context: true,
            requested_context: Some("Include nearby assignment and call sites for this value.".to_string()),
        };
    }

    if packet.rule_id == "POL-EXEC-001" && token.contains("subprocess") {
        return MockReviewPlan {
            confirmed: false,
            confidence: 0.41,
            false_positive_reason: Some(
                "Command execution was found, but usage details are required before confirm/reject."
                    .to_string(),
            ),
            needs_more_context: true,
            requested_context: Some(
                "Capture command arguments and whether input is user-controlled.".to_string(),
            ),
        };
    }

    if deterministic_bucket(packet) % 11 == 0 {
        return MockReviewPlan {
            confirmed: false,
            confidence: 0.49,
            false_positive_reason: Some(
                "The deterministic mock model is flagging low-confidence ambiguity."
                    .to_string(),
            ),
            needs_more_context: true,
            requested_context: Some("Additional context from neighboring lines is required.".to_string()),
        };
    }

    MockReviewPlan {
        confirmed: true,
        confidence: 0.82,
        false_positive_reason: None,
        needs_more_context: false,
        requested_context: None,
    }
}

fn deterministic_bucket(packet: &ContextPacket) -> u32 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in packet.cluster_id.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    for byte in packet.rule_id.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash % 997) as u32
}

fn mock_explanation(packet: &ContextPacket, plan: &MockReviewPlan) -> String {
    let base = if plan.confirmed {
        "Mock review suggests this is likely a real issue."
    } else if plan.needs_more_context {
        "Mock review could not fully validate this finding yet."
    } else {
        "Mock review marked this as a likely false positive."
    };

    format!(
        "[Mock review] Rule {} matched token '{}' in `{}`. {} Enable live review with GEMINI_API_KEY for a real assessment.",
        packet.rule_id,
        packet.matched_token,
        packet.file,
        base,
    )
}

fn mock_remediation(packet: &ContextPacket) -> String {
    packet
        .policy_hints
        .first()
        .map(|h| {
            // Strip inline compliance refs for a cleaner remediation line.
            let s = if let Some(pos) = h.find("; satisfies") {
                &h[..pos]
            } else {
                h.as_str()
            };
            let s = if let Some(pos) = s.rfind('(') {
                let suffix = &s[pos..];
                if suffix.contains("OWASP") || suffix.contains("NIST") || suffix.contains("CWE") {
                    s[..pos].trim_end_matches([' ', ';', ',']).to_string()
                } else {
                    s.to_string()
                }
            } else {
                s.to_string()
            };
            s.trim().to_string()
        })
        .unwrap_or_else(|| "Review and remediate according to policy guidelines.".to_string())
}

fn mock_adjacent_faults(packet: &ContextPacket) -> Vec<String> {
    packet
        .adjacent_bug_checks
        .first()
        .map(|c| vec![c.clone()])
        .unwrap_or_default()
}

// ── Gemini provider ───────────────────────────────────────────────────────────

/// Live provider calling the Google Gemini chat completions API (OpenAI-compatible).
///
/// # Configuration
/// - `GEMINI_API_KEY`  — required; API key for authentication.
/// - `GEMINI_MODEL`    — optional; defaults to `gemini-2.0-flash`.
/// - `GEMINI_BASE_URL` — optional; defaults to
///                       `https://generativelanguage.googleapis.com/v1beta/openai`.
///
/// Uses synchronous HTTP via `ureq`. No async runtime needed.
///
/// If the API call or response parsing fails, a failure result is returned
/// (confirmed=false, confidence=0.0) rather than silently falling back to mock.
pub struct GeminiProvider {
    api_key: String,
    model: String,
    base_url: String,
}

impl GeminiProvider {
    /// Create a provider from environment variables.
    /// Returns `None` if `GEMINI_API_KEY` is unset or empty.
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("GEMINI_API_KEY").ok()?;
        if api_key.is_empty() {
            return None;
        }
        let model = std::env::var("GEMINI_MODEL")
            .unwrap_or_else(|_| "gemini-2.5-flash".to_string());
        let base_url = std::env::var("GEMINI_BASE_URL")
            .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta/openai".to_string());
        Some(GeminiProvider { api_key, model, base_url })
    }
}

impl ReviewProvider for GeminiProvider {
    fn review(&self, packet: &ContextPacket) -> TriageResult {
        info!(
            "Calling Gemini {} for finding {} ({})",
            self.model, packet.cluster_id, packet.rule_id
        );
        match call_gemini(packet, &self.api_key, &self.model, &self.base_url) {
            Ok(result) => result,
            Err(e) => {
                warn!("Gemini review failed for {}: {}", packet.cluster_id, e);
                review_failed_result(&packet.cluster_id, &e.to_string())
            }
        }
    }
}

/// Extract the assistant message text from an OpenAI-compatible chat completions response.
pub fn extract_chat_content(response: &serde_json::Value) -> Option<&str> {
    response["choices"][0]["message"]["content"].as_str()
}

fn call_gemini(
    packet: &ContextPacket,
    api_key: &str,
    model: &str,
    base_url: &str,
) -> anyhow::Result<TriageResult> {
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": prompt::SYSTEM_PROMPT },
            { "role": "user",   "content": prompt::build_prompt(packet) }
        ],
        "max_tokens": 1024,
        "temperature": 0.2
    });

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let response = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("content-type", "application/json")
        .send_json(&body)
        .map_err(|e| anyhow::anyhow!("HTTP error: {}", e))?;

    let parsed: serde_json::Value = response
        .into_json()
        .map_err(|e| anyhow::anyhow!("Failed to parse response body: {}", e))?;

    let text = extract_chat_content(&parsed)
        .ok_or_else(|| anyhow::anyhow!("No content in response choices[0].message.content: {:?}", parsed))?;

    validator::parse_response(&packet.cluster_id, text)
        .map_err(|e| anyhow::anyhow!("Response validation failed: {}", e))
}

fn review_failed_result(cluster_id: &str, error: &str) -> TriageResult {
    TriageResult {
        cluster_id: cluster_id.to_string(),
        confirmed: false,
        confidence: 0.0,
        severity_adjustment: None,
        false_positive_reason: None,
        hidden_adjacent_faults: vec![],
        explanation: format!("Review failed: {}", error),
        remediation: "Manual review required — automated review could not complete.".to_string(),
        needs_more_context: false,
        requested_context: None,
        is_mock: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that `extract_chat_content` correctly pulls text from an
    /// OpenAI-compatible chat completions response shape (used by Gemini).
    #[test]
    fn test_extract_chat_content_from_openai_shape() {
        let response = serde_json::json!({
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "model": "gemini-2.0-flash",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "{\"confirmed\": true, \"confidence\": 0.9}"
                },
                "finish_reason": "stop"
            }],
            "usage": { "prompt_tokens": 50, "completion_tokens": 20, "total_tokens": 70 }
        });

        let text = extract_chat_content(&response);
        assert!(text.is_some(), "should extract content from choices[0].message.content");
        assert!(text.unwrap().contains("confirmed"));
    }

    #[test]
    fn test_extract_chat_content_missing_choices_returns_none() {
        let response = serde_json::json!({ "error": "model not found" });
        assert!(extract_chat_content(&response).is_none());
    }

    #[test]
    fn test_extract_chat_content_empty_choices_returns_none() {
        let response = serde_json::json!({ "choices": [] });
        assert!(extract_chat_content(&response).is_none());
    }

    fn mock_packet(rule_id: &str, token: &str, file: &str) -> ContextPacket {
        ContextPacket {
            cluster_id: "cluster-1".to_string(),
            rule_id: rule_id.to_string(),
            title: "Mocked finding".to_string(),
            category: "test".to_string(),
            severity: "high".to_string(),
            file: file.to_string(),
            language: "python".to_string(),
            start_line: 1,
            end_line: 1,
            matched_token: token.to_string(),
            match_reason: "mocked reason".to_string(),
            snippet: String::new(),
            policy_hints: vec![],
            adjacent_bug_checks: vec![],
        }
    }

    #[test]
    fn test_mock_provider_marks_sql_findings_for_context() {
        let provider = MockProvider;
        let packet = mock_packet(
            "POL-SQL-001",
            "SELECT * FROM users WHERE id = {user_id}",
            "samples/src/example.py",
        );
        let result = provider.review(&packet);

        assert!(!result.confirmed, "expected context-driven rejection");
        assert!(result.needs_more_context);
        assert!(result.false_positive_reason.is_some());
    }

    #[test]
    fn test_mock_provider_confirms_strong_secret_match() {
        let provider = MockProvider;
        let packet = mock_packet("POL-SECRET-001", "AKIAIOSFODNN7EXAMPLE", "samples/src/example.py");
        let result = provider.review(&packet);

        assert!(result.confirmed);
        assert!(!result.needs_more_context);
        assert!(result.explanation.contains("Rule POL-SECRET-001"));
    }
}
