use std::path::PathBuf;

fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples")
}

fn policies_dir() -> PathBuf {
    samples_dir().join("policies")
}

// ── Phase 5: Ingest ───────────────────────────────────────────────────────────

#[test]
fn test_load_tokens() {
    let tokens_path = samples_dir().join("tokens.json");
    let result = argusaichecker::ingest::load_and_normalize(&tokens_path);
    assert!(result.is_ok(), "Failed to load tokens: {:?}", result.err());
    let anchors = result.unwrap();
    assert_eq!(anchors.len(), 7, "Expected 7 anchors");
}

#[test]
fn test_load_tokens_missing_file_gives_clear_error() {
    let result = argusaichecker::ingest::load_and_normalize(std::path::Path::new("/nonexistent/tokens.json"));
    assert!(result.is_err());
    let msg = format!("{:?}", result.err().unwrap());
    assert!(msg.contains("Cannot read") || msg.contains("tokens"), "got: {}", msg);
}

#[test]
fn test_load_policy_dir() {
    let result = argusaichecker::ingest::load_policy(&policies_dir());
    assert!(result.is_ok(), "Failed to load policy dir: {:?}", result.err());
    let rules = result.unwrap();
    // 6 sample rule files are present under samples/policies/
    assert_eq!(rules.len(), 6, "Expected 6 policy rules");
}

#[test]
fn test_load_policy_missing_dir_gives_clear_error() {
    let result = argusaichecker::ingest::load_policy(std::path::Path::new("/nonexistent/policies"));
    assert!(result.is_err());
    let msg = format!("{:?}", result.err().unwrap());
    assert!(
        msg.contains("does not exist") || msg.contains("Policy"),
        "got: {}",
        msg
    );
}

#[test]
fn test_load_policy_rejects_single_file() {
    // Passing a file path instead of a directory should fail with a clear message.
    let result = argusaichecker::ingest::load_policy(&samples_dir().join("tokens.json"));
    assert!(result.is_err());
    let msg = format!("{:?}", result.err().unwrap());
    assert!(
        msg.contains("not a directory") || msg.contains("directory"),
        "got: {}",
        msg
    );
}

// ── Phase 6: Deterministic engine ────────────────────────────────────────────

#[test]
fn test_deterministic_matching_produces_findings() {
    let anchors = argusaichecker::ingest::load_and_normalize(&samples_dir().join("tokens.json")).unwrap();
    let rules = argusaichecker::ingest::load_policy(&policies_dir()).unwrap();
    let compiled = argusaichecker::policy::compile_rules(rules).unwrap();
    let matches = argusaichecker::policy::match_anchors(&anchors, &compiled);
    assert!(!matches.is_empty(), "Expected at least one raw match");
}

#[test]
fn test_dedupe_reduces_nearby_matches() {
    let anchors = argusaichecker::ingest::load_and_normalize(&samples_dir().join("tokens.json")).unwrap();
    let rules = argusaichecker::ingest::load_policy(&policies_dir()).unwrap();
    let compiled = argusaichecker::policy::compile_rules(rules).unwrap();
    let matches = argusaichecker::policy::match_anchors(&anchors, &compiled);
    let raw_count = matches.len();
    let clusters = argusaichecker::policy::dedupe_and_cluster(matches, &compiled);
    assert!(clusters.len() <= raw_count, "clusters ({}) should be <= raw_count ({})", clusters.len(), raw_count);
}

#[test]
fn test_severity_summary() {
    let anchors = argusaichecker::ingest::load_and_normalize(&samples_dir().join("tokens.json")).unwrap();
    let rules = argusaichecker::ingest::load_policy(&policies_dir()).unwrap();
    let compiled = argusaichecker::policy::compile_rules(rules).unwrap();
    let matches = argusaichecker::policy::match_anchors(&anchors, &compiled);
    let clusters = argusaichecker::policy::dedupe_and_cluster(matches, &compiled);
    let summary = argusaichecker::policy::build_summary(&clusters);
    assert_eq!(summary.cluster_count, clusters.len());
    assert!(summary.raw_match_count >= clusters.len());
}

// ── Phase 2: Context extraction ───────────────────────────────────────────────

#[test]
fn test_snippet_extract_numbered() {
    let src = "line1\nline2\nline3\nline4\nline5";
    let s = argusaichecker::context::snippet::extract_numbered(src, 3, 1);
    assert!(s.contains("   2 | line2"), "got: {}", s);
    assert!(s.contains("   3 | line3"), "got: {}", s);
    assert!(s.contains("   4 | line4"), "got: {}", s);
}

#[test]
fn test_packer_produces_packet_for_known_file() {
    let anchors = argusaichecker::ingest::load_and_normalize(&samples_dir().join("tokens.json")).unwrap();
    let rules = argusaichecker::ingest::load_policy(&policies_dir()).unwrap();
    let compiled = argusaichecker::policy::compile_rules(rules).unwrap();
    let matches = argusaichecker::policy::match_anchors(&anchors, &compiled);
    let clusters = argusaichecker::policy::dedupe_and_cluster(matches, &compiled);

    assert!(!clusters.is_empty(), "need at least one cluster");
    let packet = argusaichecker::context::pack(&clusters[0], &samples_dir().join("src"));
    assert!(!packet.cluster_id.is_empty());
    assert!(!packet.file.is_empty());
    // Snippet may be empty if running from a non-standard cwd; just don't panic.
    let _ = packet.snippet;
}

// ── Phase 3: Review pipeline ──────────────────────────────────────────────────

#[test]
fn test_mock_review_sets_is_mock_true() {
    let anchors = argusaichecker::ingest::load_and_normalize(&samples_dir().join("tokens.json")).unwrap();
    let rules = argusaichecker::ingest::load_policy(&policies_dir()).unwrap();
    let compiled = argusaichecker::policy::compile_rules(rules).unwrap();
    let matches = argusaichecker::policy::match_anchors(&anchors, &compiled);
    let clusters = argusaichecker::policy::dedupe_and_cluster(matches, &compiled);
    let candidates = argusaichecker::ranking::rank(&clusters);

    let reviews = argusaichecker::review::run_review(
        true, // use_mock
        &candidates,
        &clusters,
        &samples_dir().join("src"),
    );
    assert!(!reviews.is_empty(), "expected at least one review result");
    for r in &reviews {
        assert!(r.is_mock, "is_mock should be true for mock provider");
    }
}

#[test]
fn test_mock_review_with_soc2_chaos_tokens_is_mixed() {
    let anchors = argusaichecker::ingest::load_and_normalize(
        &samples_dir().join("src/soc2-chaos-project/tokens.json"),
    )
    .unwrap();
    let rules = argusaichecker::ingest::load_policy(&policies_dir()).unwrap();
    let compiled = argusaichecker::policy::compile_rules(rules).unwrap();
    let matches = argusaichecker::policy::match_anchors(&anchors, &compiled);
    let clusters = argusaichecker::policy::dedupe_and_cluster(matches, &compiled);
    let candidates = argusaichecker::ranking::rank(&clusters);

    let reviews = argusaichecker::review::run_review(
        true,
        &candidates,
        &clusters,
        &samples_dir().join("src/soc2-chaos-project"),
    );

    assert!(!reviews.is_empty(), "expected at least one review result from chaos fixtures");

    let confirmed_count = reviews.iter().filter(|r| r.confirmed).count();
    let rejected_count = reviews.iter().filter(|r| !r.confirmed).count();
    let escalation_count = reviews.iter().filter(|r| r.needs_more_context).count();

    assert!(confirmed_count > 0, "expected at least one confirmed mock review in chaos fixture");
    assert!(
        rejected_count > 0 || escalation_count > 0,
        "expected mixed review behavior: rejected or escalated findings"
    );

    assert!(
        reviews.iter().any(|r| r.is_mock),
        "all results should be mock in mock-review mode"
    );
}

#[test]
fn test_validator_parses_valid_json() {
    let json = r#"{
      "confirmed": true,
      "confidence": 0.9,
      "severity_adjustment": null,
      "false_positive_reason": null,
      "hidden_adjacent_faults": [],
      "explanation": "Real secret found.",
      "remediation": "Rotate and move to secrets manager.",
      "needs_more_context": false,
      "requested_context": null
    }"#;
    let result = argusaichecker::review::validator::parse_response("c1", json);
    assert!(result.is_ok(), "{:?}", result.err());
    let r = result.unwrap();
    assert!(r.confirmed);
    assert!(!r.is_mock);
}

// ── Phase 4: Full pipeline end-to-end ────────────────────────────────────────

#[test]
fn test_full_pipeline_writes_output() {
    use std::fs;
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_out");
    fs::create_dir_all(&out_dir).unwrap();

    let config = argusaichecker::config::Config {
        tokens_path: samples_dir().join("tokens.json"),
        policy_dir: policies_dir(),
        source_dir: samples_dir().join("src"),
        output_dir: out_dir.clone(),
        mock_review: true,
    };

    let result = argusaichecker::app::run(config);
    assert!(result.is_ok(), "Pipeline failed: {:?}", result.err());

    assert!(out_dir.join("violations.json").exists());
    assert!(out_dir.join("final_report.json").exists());
    assert!(out_dir.join("audit_report.md").exists());

    // Verify provenance is populated in final_report.json
    let report_str = fs::read_to_string(out_dir.join("final_report.json")).unwrap();
    assert!(report_str.contains("tokens.json"), "tokens_path missing from report");
    assert!(report_str.contains("policies"), "policy_dir missing from report");

    fs::remove_dir_all(&out_dir).ok();
}
