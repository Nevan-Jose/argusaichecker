use crate::schemas::review::TriageResult;
use crate::schemas::violations::{SeveritySummary, ViolationCluster};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub fn write(
    path: &Path,
    summary: &SeveritySummary,
    clusters: &[ViolationCluster],
    reviews: &[TriageResult],
) -> Result<()> {
    let review_map: HashMap<&str, &TriageResult> =
        reviews.iter().map(|r| (r.cluster_id.as_str(), r)).collect();

    let mut out = String::new();

    // ── 1. Title ─────────────────────────────────────────────────────────────
    out.push_str("# Argus Security Audit Report\n\n");
    out.push_str("> This report was generated automatically. ");
    out.push_str("Preliminary review results are marked as such and should be verified by a developer or security engineer before acting on them.\n\n");
    out.push_str("---\n\n");

    // ── 2. Executive Summary ──────────────────────────────────────────────────
    out.push_str("## Executive Summary\n\n");
    out.push_str(&format!(
        "The scan identified **{} security issue{}** across **{} location{}** in the codebase.\n\n",
        summary.cluster_count,
        if summary.cluster_count == 1 { "" } else { "s" },
        summary.by_file.len(),
        if summary.by_file.len() == 1 { "" } else { "s" },
    ));

    // Severity counts in human order
    let sev_order = ["critical", "high", "medium", "low", "info"];
    let has_any = sev_order.iter().any(|s| summary.by_severity.contains_key(*s));
    if has_any {
        out.push_str("| Severity | Issues Found |\n");
        out.push_str("|---|---|\n");
        for sev in &sev_order {
            if let Some(count) = summary.by_severity.get(*sev) {
                out.push_str(&format!("| {} | {} |\n", severity_label(sev), count));
            }
        }
        out.push('\n');
    }

    // Plain-English summary of highest-risk items
    let critical_count = summary.by_severity.get("critical").copied().unwrap_or(0);
    let high_count = summary.by_severity.get("high").copied().unwrap_or(0);
    if critical_count > 0 {
        out.push_str(&format!(
            "**{} critical issue{}** require immediate attention — these represent the highest risk to your application and data.\n\n",
            critical_count,
            if critical_count == 1 { "" } else { "s" },
        ));
    }
    if high_count > 0 {
        out.push_str(&format!(
            "**{} high-severity issue{}** should be addressed as soon as possible.\n\n",
            high_count,
            if high_count == 1 { "" } else { "s" },
        ));
    }

    // ── 3. Priority Actions ───────────────────────────────────────────────────
    out.push_str("---\n\n## Priority Actions\n\n");
    out.push_str("Address these issues first:\n\n");
    let top_n = clusters.iter().take(3).enumerate();
    for (i, c) in top_n {
        let first_hint = c.policy_hints.first().map(|h| strip_compliance_refs(h)).unwrap_or_default();
        out.push_str(&format!(
            "{}. **{}** — `{}` line {}.{}\n",
            i + 1,
            c.title,
            c.file,
            c.start_line,
            if first_hint.is_empty() {
                String::new()
            } else {
                format!(" {}", first_hint)
            },
        ));
    }
    out.push('\n');

    // ── 4. Findings ───────────────────────────────────────────────────────────
    out.push_str("---\n\n## Findings\n\n");

    for (idx, c) in clusters.iter().enumerate() {
        let review = review_map.get(c.cluster_id.as_str()).copied();

        // Heading
        out.push_str(&format!(
            "### Finding {}: {} {}\n\n",
            idx + 1,
            severity_badge(c.severity.to_string().as_str()),
            c.title,
        ));

        // Location
        out.push_str("**Location**\n\n");
        let line_range = if c.start_line == c.end_line {
            format!("line {}", c.start_line)
        } else {
            format!("lines {}–{}", c.start_line, c.end_line)
        };
        out.push_str(&format!("- File: `{}`\n", c.file));
        out.push_str(&format!("- {}\n\n", line_range));

        // What Is Wrong
        out.push_str("**What Is Wrong**\n\n");
        out.push_str(&format!("{}\n\n", what_is_wrong(c)));

        // Matched Evidence (from first raw match)
        if let Some(first_match) = c.matches.first() {
            out.push_str("**Matched Evidence**\n\n");
            out.push_str(&format!(
                "The scanner found `{}` — {}\n\n",
                truncate(&first_match.matched_token, 80),
                first_match.reason,
            ));
        }

        // Why It Matters
        out.push_str("**Why It Matters**\n\n");
        out.push_str(&format!("{}\n\n", why_it_matters(c)));

        // What To Fix
        out.push_str("**What To Fix**\n\n");
        let hints = hints_as_actions(&c.policy_hints);
        if hints.is_empty() {
            if let Some(r) = review {
                if !r.remediation.is_empty() && r.remediation != "Review and remediate according to policy hints." {
                    out.push_str(&format!("- {}\n", r.remediation));
                }
            }
        } else {
            for h in &hints {
                out.push_str(&format!("- {}\n", h));
            }
        }
        out.push('\n');

        // Compliance References
        let refs = extract_compliance_refs(&c.policy_hints);
        if !refs.is_empty() {
            out.push_str("**Compliance References**\n\n");
            out.push_str(&format!("{}\n\n", refs.join(" · ")));
        }

        // Nearby Checks
        if !c.adjacent_bug_checks.is_empty() {
            out.push_str("**Nearby Risks to Check**\n\n");
            for check in &c.adjacent_bug_checks {
                out.push_str(&format!("- {}\n", check));
            }
            out.push('\n');
        }

        // AI / Preliminary Review
        out.push_str("**Preliminary Review**\n\n");
        if let Some(r) = review {
            let status = if r.confirmed { "Issue confirmed" } else { "Possible false positive" };
            let confidence_pct = (r.confidence * 100.0).round() as u32;
            let review_label = if r.is_mock {
                "⚠️ *This is a mock/offline review for demo purposes — not a real model judgment.*"
            } else if r.confidence == 0.0 && r.explanation.starts_with("Review failed") {
                "⚠️ *Automated review failed. Manual inspection required.*"
            } else {
                "🤖 *This assessment was produced by an AI model. Verify with a qualified engineer before acting.*"
            };
            out.push_str(&format!("> {}\n>\n", review_label));
            out.push_str(&format!("> **Status:** {}  \n", status));
            if r.confidence > 0.0 {
                out.push_str(&format!("> **Confidence:** {}%  \n", confidence_pct));
            }
            let skip_explanation = r.explanation.starts_with("[Mock review]")
                || r.explanation.starts_with("Review failed");
            if !r.explanation.is_empty() && !skip_explanation {
                out.push_str(&format!("> **Notes:** {}\n", r.explanation));
            }
            if let Some(fp_reason) = &r.false_positive_reason {
                out.push_str(&format!("> **Possible False Positive Reason:** {}\n", fp_reason));
            }
            if !r.hidden_adjacent_faults.is_empty() {
                out.push_str(">\n> **Potentially Related Issues Nearby:**\n");
                for f in &r.hidden_adjacent_faults {
                    out.push_str(&format!("> - {}\n", f));
                }
            }
        } else {
            out.push_str("> This finding has not yet been reviewed.\n");
        }
        out.push('\n');

        out.push_str("---\n\n");
    }

    // ── 5. Footer ─────────────────────────────────────────────────────────────
    out.push_str("*Report generated by Argus. Findings are deterministic pattern matches ");
    out.push_str("and should be confirmed by a qualified engineer before remediation.*\n");

    std::fs::write(path, out)?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn severity_label(sev: &str) -> &'static str {
    match sev {
        "critical" => "🔴 Critical",
        "high"     => "🟠 High",
        "medium"   => "🟡 Medium",
        "low"      => "🟢 Low",
        "info"     => "ℹ️ Info",
        _          => "Unknown",
    }
}

fn severity_badge(sev: &str) -> &'static str {
    match sev {
        "critical" => "🔴",
        "high"     => "🟠",
        "medium"   => "🟡",
        "low"      => "🟢",
        "info"     => "ℹ️",
        _          => "⚪",
    }
}

fn what_is_wrong(c: &ViolationCluster) -> String {
    match c.category.to_string().as_str() {
        "secrets" => format!(
            "A secret, credential, or API key appears to be written directly into the source code in `{}`. \
             Anyone who can read this file — including version control history — can see this value.",
            c.file
        ),
        "crypto" => format!(
            "The code in `{}` uses a weak or outdated security algorithm. \
             Algorithms like MD5, SHA-1, DES, or RC4 are no longer considered safe and can be broken by modern computers.",
            c.file
        ),
        "injection" => format!(
            "The code in `{}` builds a database query or command by inserting user-supplied text directly into it. \
             An attacker could manipulate this text to access or alter data they should not be able to reach.",
            c.file
        ),
        "execution" => format!(
            "The code in `{}` runs operating system commands, and the input to those commands may be controllable by an external user. \
             This can allow an attacker to run arbitrary commands on the server.",
            c.file
        ),
        "auth" => format!(
            "A security or authentication check in `{}` has been disabled or set to always pass. \
             This means the application may accept requests that should be rejected — such as expired tokens or forged credentials.",
            c.file
        ),
        "deserialization" => format!(
            "The code in `{}` converts raw data from an external source into executable objects without verifying that the data is safe. \
             A malicious payload could cause unexpected or harmful behavior when processed.",
            c.file
        ),
        "compliance" => format!(
            "Sensitive information may be written to log files or output streams in `{}`. \
             Log data is often stored and shared broadly, making this a data exposure risk.",
            c.file
        ),
        "other" => format!(
            "A potentially risky operation was detected in `{}`. \
             Review the matched evidence and remediation steps below for details.",
            c.file
        ),
        _ => format!("A security issue was detected in `{}`.", c.file),
    }
}

fn why_it_matters(c: &ViolationCluster) -> String {
    let severity_intro = match c.severity.to_string().as_str() {
        "critical" => "This is a **critical-severity** issue. Exploitation could cause immediate and severe harm, such as unauthorized access to systems or exposure of sensitive data.",
        "high"     => "This is a **high-severity** issue. It represents a significant security risk that could be exploited to compromise data, accounts, or system integrity.",
        "medium"   => "This is a **medium-severity** issue. It may not be immediately exploitable on its own, but could be combined with other weaknesses to cause harm.",
        "low"      => "This is a **low-severity** issue. The risk is limited in most contexts, but should still be addressed to maintain a strong security posture.",
        _          => "This issue may represent a security or compliance concern worth reviewing.",
    };
    let category_detail = match c.category.to_string().as_str() {
        "secrets"         => " Exposed credentials can be used to access cloud services, databases, or APIs without authorization — often within minutes of code being published.",
        "crypto"          => " Weak algorithms can be reversed or forged with readily available tools, undermining the protection they were intended to provide.",
        "injection"       => " SQL injection is one of the most commonly exploited vulnerabilities. It can lead to data theft, data loss, or complete database compromise.",
        "execution"       => " Command injection can give an attacker full control over the server, allowing them to read files, install software, or disrupt services.",
        "auth"            => " Bypassed authentication checks allow attackers to impersonate users, reuse expired tokens, or access resources without valid credentials.",
        "deserialization" => " Unsafe deserialization can lead to remote code execution — one of the most severe vulnerability classes.",
        "compliance"      => " Logging sensitive data may violate privacy regulations (such as GDPR or HIPAA) and expose credentials or personal information to anyone with log access.",
        _                 => "",
    };
    format!("{}{}", severity_intro, category_detail)
}

/// Convert policy_hints to action items, stripping inline compliance ref suffixes.
fn hints_as_actions(hints: &[String]) -> Vec<String> {
    hints.iter().map(|h| strip_compliance_refs(h)).filter(|s| !s.is_empty()).collect()
}

/// Remove trailing parenthetical compliance refs like "(OWASP ASVS V6.4, NIST SP 800-53 IA-5)"
/// and suffix clauses like "; satisfies OWASP ...".
fn strip_compliance_refs(s: &str) -> String {
    // Remove trailing "(OWASP..." style parenthetical
    let s = if let Some(pos) = s.rfind('(') {
        let suffix = &s[pos..];
        if suffix.contains("OWASP") || suffix.contains("NIST") || suffix.contains("CWE") || suffix.contains("PCI") {
            s[..pos].trim_end_matches([' ', ';', ',']).to_string()
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    };
    // Remove "; satisfies ..." suffix
    let s = if let Some(pos) = s.find("; satisfies") {
        s[..pos].to_string()
    } else {
        s
    };
    s.trim().to_string()
}

/// Extract short compliance reference tokens from policy_hints text.
/// Looks for OWASP ASVS, CWE-NNN, NIST SP xxx, and PCI DSS occurrences
/// and returns only the compact ID portion (e.g. "OWASP ASVS V6.4",
/// "CWE-798", "NIST SP 800-53 IA-5").
fn extract_compliance_refs(hints: &[String]) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();
    let combined = hints.join(" ");

    extract_ref_tokens(&combined, "OWASP ASVS", &mut refs);
    extract_ref_tokens(&combined, "CWE-", &mut refs);
    extract_ref_tokens(&combined, "NIST SP", &mut refs);
    if combined.contains("PCI DSS") && !refs.contains(&"PCI DSS".to_string()) {
        refs.push("PCI DSS".to_string());
    }

    // Remove any entry that is a substring of a longer entry already present.
    let snapshot = refs.clone();
    refs.retain(|r| !snapshot.iter().any(|other| other != r && other.contains(r.as_str())));

    refs
}

/// For each occurrence of `keyword` in `text`, extract a compact reference
/// token by consuming only words that look like part of a reference ID
/// (version numbers, control identifiers, "and", the known framework words).
/// Results are deduplicated and appended to `out`.
fn extract_ref_tokens(text: &str, keyword: &str, out: &mut Vec<String>) {
    let mut search = text;
    while let Some(pos) = search.find(keyword) {
        let rest = &search[pos..];
        let token = collect_ref_words(rest);
        if !token.is_empty() && !out.contains(&token) {
            out.push(token);
        }
        search = &search[pos + keyword.len()..];
    }
}

/// Starting at the beginning of `s` (which begins with a known keyword),
/// collect words that are plausibly part of a reference identifier and
/// return the trimmed result. Stops as soon as a word does not match
/// the reference vocabulary.
fn collect_ref_words(s: &str) -> String {
    // Words that are allowed to continue a reference token.
    const REF_WORDS: &[&str] = &[
        "OWASP", "ASVS", "CWE", "NIST", "SP", "PCI", "DSS", "and",
    ];

    let mut result = String::new();
    let mut first = true;
    for word in s.split_whitespace() {
        // Strip trailing punctuation for classification only.
        let clean: String = word.chars().take_while(|c| !matches!(c, ',' | ';' | ')' | '(')).collect();
        if clean.is_empty() {
            break;
        }
        // A "reference word" is either an explicit keyword or a token whose
        // characters are all uppercase letters, digits, dashes, or dots —
        // this covers version tokens (V6.4), control IDs (IA-5, SI-10,
        // 800-53, 131A) while excluding plain English words like "Rotate".
        let is_ref_word = REF_WORDS.contains(&clean.as_str())
            || clean.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' || c == '.');

        if is_ref_word {
            if !first {
                result.push(' ');
            }
            // Append the clean version (without trailing punctuation).
            result.push_str(&clean);
            first = false;
            // If the original word had a stopping character right after the
            // clean part, we are done.
            if word.len() > clean.len() {
                break;
            }
        } else {
            break;
        }
    }
    result.trim().to_string()
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
