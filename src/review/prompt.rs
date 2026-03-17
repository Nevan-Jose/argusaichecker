use crate::context::ContextPacket;

/// System prompt that instructs the model to return strict structured JSON only.
pub const SYSTEM_PROMPT: &str = r#"You are a security code-review assistant.

Your job is to review a specific code finding and return a structured JSON assessment.

Rules:
- Return ONLY a single JSON object. No prose, no markdown, no explanation outside the JSON.
- All fields are required. Use null for optional fields when not applicable.
- Do not invent evidence. Base your judgment only on what is shown.
- "confirmed" means the pattern likely represents a real exploitable issue in this context.
- "confidence" is a float between 0.0 and 1.0.
- "severity_adjustment" is one of: "info", "low", "medium", "high", "critical", or null.
- "hidden_adjacent_faults" is a list of short strings describing other issues you noticed nearby.
- "needs_more_context" is true only if you genuinely cannot assess without additional code.

Return this exact JSON shape:
{
  "confirmed": <bool>,
  "confidence": <float>,
  "severity_adjustment": <string or null>,
  "false_positive_reason": <string or null>,
  "hidden_adjacent_faults": [<string>, ...],
  "explanation": <string>,
  "remediation": <string>,
  "needs_more_context": <bool>,
  "requested_context": <string or null>
}"#;

/// Build the user-turn message for a single finding review.
pub fn build_prompt(packet: &ContextPacket) -> String {
    let mut msg = String::new();

    msg.push_str("## Security Finding Review Request\n\n");
    msg.push_str(&format!("**Rule:** {}\n", packet.rule_id));
    msg.push_str(&format!("**Title:** {}\n", packet.title));
    msg.push_str(&format!("**Category:** {}\n", packet.category));
    msg.push_str(&format!("**Severity:** {}\n", packet.severity));
    msg.push_str(&format!("**File:** `{}`\n", packet.file));
    msg.push_str(&format!(
        "**Location:** lines {}–{}\n",
        packet.start_line, packet.end_line
    ));
    msg.push_str(&format!("**Language:** {}\n\n", packet.language));

    msg.push_str("### Matched Evidence\n\n");
    msg.push_str(&format!("Token: `{}`\n", packet.matched_token));
    msg.push_str(&format!("Reason: {}\n\n", packet.match_reason));

    if !packet.snippet.is_empty() {
        msg.push_str("### Code Snippet\n\n");
        msg.push_str("```");
        msg.push_str(&packet.language);
        msg.push('\n');
        msg.push_str(&packet.snippet);
        msg.push_str("\n```\n\n");
    }

    if !packet.policy_hints.is_empty() {
        msg.push_str("### Policy Hints\n\n");
        for hint in &packet.policy_hints {
            msg.push_str(&format!("- {}\n", hint));
        }
        msg.push('\n');
    }

    if !packet.adjacent_bug_checks.is_empty() {
        msg.push_str("### Adjacent Bug Checks\n\n");
        for check in &packet.adjacent_bug_checks {
            msg.push_str(&format!("- {}\n", check));
        }
        msg.push('\n');
    }

    msg.push_str("Respond with the JSON assessment only.");
    msg
}
