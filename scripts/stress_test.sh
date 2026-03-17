#!/usr/bin/env bash
# Argus stress-test matrix runner
# Usage: ./scripts/stress_test.sh
# Runs 18 test cases and prints a pass/fail summary table.

set -euo pipefail

BINARY="./target/release/argus"
TOKENS_DIR="test data/tokens"
POLICIES_DIR="test data/policies"
SRC_DIR="test data/src"
SAMPLES_POLICY="samples/policy.json"
OUT_BASE="out/stress"

PASS=0
FAIL=0
UNEXPECTED=0
declare -a RESULTS

run_case() {
    local case_num="$1"
    local case_name="$2"
    local expect="$3"   # "pass" or "fail"
    local tokens="$4"
    local policy="$5"
    local out_dir="${OUT_BASE}/case_$(printf '%02d' "$case_num")"

    mkdir -p "$out_dir"

    # Run and capture output + exit code
    local output
    local exit_code=0
    output=$("$BINARY" \
        --tokens "$tokens" \
        --policy "$policy" \
        --source-dir "$SRC_DIR" \
        --output-dir "$out_dir" \
        --mock-review 2>&1) || exit_code=$?

    local actual="pass"
    [[ $exit_code -ne 0 ]] && actual="fail"

    local status
    if [[ "$actual" == "$expect" ]]; then
        status="PASS"
        PASS=$((PASS + 1))
    else
        status="FAIL (unexpected: expected=$expect got=$actual)"
        FAIL=$((FAIL + 1))
        UNEXPECTED=$((UNEXPECTED + 1))
    fi

    # Count findings/clusters from violations.json if it exists
    local findings="-"
    local clusters="-"
    local outputs="-"
    if [[ -f "$out_dir/violations.json" ]]; then
        findings=$(python3 -c "import json,sys; d=json.load(open('$out_dir/violations.json')); print(d['summary']['raw_match_count'])" 2>/dev/null || echo "?")
        clusters=$(python3 -c "import json,sys; d=json.load(open('$out_dir/violations.json')); print(len(d['findings']))" 2>/dev/null || echo "?")
        outputs=$(ls "$out_dir" | tr '\n' ' ')
    fi

    RESULTS+=("$(printf '%-3s | %-50s | %-8s | %-8s | %-6s | %-6s | %s' \
        "$case_num" "$case_name" "$expect" "$status" "$findings" "$clusters" "$outputs")")
    echo "[$case_num] $case_name — $status"
    if [[ "$status" != "PASS" ]]; then
        echo "    Exit code: $exit_code"
        echo "    Output: $(echo "$output" | head -5)"
    fi
}

# ── Build ──────────────────────────────────────────────────────────────────────
echo "Building argus..."
cargo build --release 2>&1 | grep -E 'error|warning.*unused|Finished|Compiling argus' || true
echo ""

# ── Valid token tests (expected to produce output) ────────────────────────────
echo "=== Running 18 stress-test cases ==="
echo ""

# Case 1: Full matrix — all categories
run_case 1 "valid_full_matrix + main_policy" "pass" \
    "$TOKENS_DIR/valid_full_matrix.json" "$SAMPLES_POLICY"

# Case 2: Empty anchors — no findings
run_case 2 "valid_empty + main_policy" "pass" \
    "$TOKENS_DIR/valid_empty.json" "$SAMPLES_POLICY"

# Case 3: Noise-only tokens — no findings expected
run_case 3 "valid_noise_only + main_policy" "pass" \
    "$TOKENS_DIR/valid_noise_only.json" "$SAMPLES_POLICY"

# Case 4: Case and path normalization
run_case 4 "valid_case_and_path_normalization + main_policy" "pass" \
    "$TOKENS_DIR/valid_case_and_path_normalization.json" "$SAMPLES_POLICY"

# Case 5: Dedupe edge cases
run_case 5 "valid_dedupe_edges + main_policy" "pass" \
    "$TOKENS_DIR/valid_dedupe_edges.json" "$SAMPLES_POLICY"

# Case 6: Path filter targets + path-filtered policy (only admin/*.py matches)
run_case 6 "valid_path_filter_targets + valid_path_filtered" "pass" \
    "$TOKENS_DIR/valid_path_filter_targets.json" "$POLICIES_DIR/valid_path_filtered.json"

# Case 7: Sparse optional fields
run_case 7 "valid_sparse_optional_fields + main_policy" "pass" \
    "$TOKENS_DIR/valid_sparse_optional_fields.json" "$SAMPLES_POLICY"

# Case 8: Weird but parseable data (line=0, unknown lang, etc.)
run_case 8 "valid_weird_but_parseable + main_policy" "pass" \
    "$TOKENS_DIR/valid_weird_but_parseable.json" "$SAMPLES_POLICY"

# Case 9: Valid tokens + empty rules — always 0 findings
run_case 9 "valid_full_matrix + valid_empty_rules" "pass" \
    "$TOKENS_DIR/valid_full_matrix.json" "$POLICIES_DIR/valid_empty_rules.json"

# ── Invalid token tests (expected to fail) ───────────────────────────────────

# Case 10: Missing required field
run_case 10 "invalid_missing_required_field + main_policy" "fail" \
    "$TOKENS_DIR/invalid_missing_required_field.json" "$SAMPLES_POLICY"

# Case 11: Wrong top-level key (uses "tokens" not "anchors")
run_case 11 "invalid_wrong_top_level_key + main_policy" "fail" \
    "$TOKENS_DIR/invalid_wrong_top_level_key.json" "$SAMPLES_POLICY"

# Case 12: Anchors is an object not an array
run_case 12 "invalid_anchors_not_array + main_policy" "fail" \
    "$TOKENS_DIR/invalid_anchors_not_array.json" "$SAMPLES_POLICY"

# Case 13: Malformed JSON (truncated)
run_case 13 "invalid_malformed + main_policy" "fail" \
    "$TOKENS_DIR/invalid_malformed.json" "$SAMPLES_POLICY"

# Case 14: Null required fields
run_case 14 "invalid_null_requireds + main_policy" "fail" \
    "$TOKENS_DIR/invalid_null_requireds.json" "$SAMPLES_POLICY"

# Case 15: Wrong types (file:int, line:str, token:array)
run_case 15 "invalid_wrong_types + main_policy" "fail" \
    "$TOKENS_DIR/invalid_wrong_types.json" "$SAMPLES_POLICY"

# ── Invalid policy tests (expected to fail) ──────────────────────────────────

# Case 16: Policy with bad regex (unclosed group)
run_case 16 "valid_full_matrix + invalid_regex" "fail" \
    "$TOKENS_DIR/valid_full_matrix.json" "$POLICIES_DIR/invalid_regex.json"

# Case 17: Policy with bad glob (unclosed bracket)
run_case 17 "valid_full_matrix + invalid_glob" "fail" \
    "$TOKENS_DIR/valid_full_matrix.json" "$POLICIES_DIR/invalid_glob.json"

# Case 18: Policy with unknown enum values
run_case 18 "valid_full_matrix + invalid_wrong_enum" "fail" \
    "$TOKENS_DIR/valid_full_matrix.json" "$POLICIES_DIR/invalid_wrong_enum.json"

# ── Summary table ──────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════"
echo "STRESS TEST RESULTS"
echo "═══════════════════════════════════════════════════════════════════════════════"
printf '%-3s | %-50s | %-8s | %-8s | %-6s | %-6s | %s\n' \
    "#" "Case" "Expected" "Status" "Matches" "Clustrs" "Outputs"
echo "────────────────────────────────────────────────────────────────────────────────"
for r in "${RESULTS[@]}"; do
    echo "$r"
done
echo "────────────────────────────────────────────────────────────────────────────────"
TOTAL=$((PASS + FAIL))
echo "Total: $TOTAL  |  Passed: $PASS  |  Failed (unexpected): $UNEXPECTED"
echo "═══════════════════════════════════════════════════════════════════════════════"
