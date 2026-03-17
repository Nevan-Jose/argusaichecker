# Data Contracts And Schemas

## Purpose

This file defines the JSON contracts and internal domain types used throughout the project.

The main goal is to lock the shapes before implementing logic.

## Contract Design Principles

- Keep schemas explicit
- Prefer required fields unless optionality is necessary
- Use enums for bounded values
- Separate external input models from internal normalized models
- Keep the review response schema strict

## External Input Contracts

### `tokens.json`

Represents Layer 1 lexical-analysis output.

Suggested fields per token/anchor record:

- `file`
- `language`
- `line`
- `column`
- `token`
- `token_kind`
- `normalized_kind`
- `ast_metadata` (optional)
- `layer1_rule_id` (optional)
- `layer1_confidence` (optional)

Purpose:

- Provide deterministic anchor points for Layer 2

### `policy.json`

Represents deterministic review rules.

Suggested fields per rule:

- `rule_id`
- `title`
- `description`
- `category`
- `severity`
- `languages`
- `path_patterns`
- `token_patterns`
- `metadata_filters`
- `dedupe_radius_lines`
- `review_required`
- `policy_hints`
- `adjacent_bug_checks`

Purpose:

- Drive deterministic matching and inform AI triage

### Optional `violations.json`

Represents precomputed deterministic findings.

Purpose:

- Allow running AI review on findings created earlier
- Support testing the AI layer independently

## Internal Domain Types

### `Layer1Anchor`

Normalized internal representation of a Layer 1 record.

Fields should include:

- stable internal id
- file path
- language
- line and column
- token text
- token kind
- normalized token kind
- optional metadata
- optional original Layer 1 labels

### `CompiledPolicyRule`

Runtime-optimized version of a policy rule.

Fields may include:

- original rule metadata
- compiled regex patterns
- compiled path globs
- normalized language filters

### `RawMatch`

Represents a direct deterministic hit before dedupe.

Fields should include:

- match id
- rule id
- title
- category
- base severity
- file path
- line and column
- matched token or evidence
- reason string
- source anchor reference

### `ViolationCluster`

Represents nearby related raw matches merged together.

Fields should include:

- cluster id
- rule id
- representative location
- list of raw matches
- merged line span
- count of merged matches
- severity summary

### `ReviewCandidate`

Represents a finding selected for AI review.

Fields should include:

- finding reference
- rank score or priority
- reason selected for review
- review mode

### `ContextPacket`

Represents the compact payload sent to the AI layer.

Fields should include:

- finding id
- rule summary
- category and severity
- file path
- language
- anchor location
- minimal snippet
- deterministic evidence
- policy hints
- adjacent bug checklist

### `TriageResult`

Represents the validated structured response from the AI.

Required fields:

- `confirmed`
- `confidence`
- `severity_adjustment`
- `false_positive_reason`
- `hidden_adjacent_faults`
- `explanation`
- `remediation`
- `needs_more_context`
- `requested_context`

### `FinalFinding`

Represents the merged deterministic + AI-reviewed result.

Fields should include:

- finding id
- source rule metadata
- deterministic evidence
- original severity
- adjusted severity
- review status
- confidence
- explanation
- hidden adjacent faults
- remediation
- provenance

### `RunSummary`

Represents overall pipeline statistics.

Fields should include:

- run timestamp
- inputs used
- raw anchor count
- raw match count
- deduped finding count
- reviewed finding count
- confirmed count
- false positive count
- escalation count
- output file paths

## Enums To Define

Recommended enums:

- `Severity`: `Info`, `Low`, `Medium`, `High`, `Critical`
- `Category`: `Secrets`, `Crypto`, `Injection`, `Execution`, `Auth`, `Compliance`, `Deserialization`, `Other`
- `ReviewStatus`: `Unreviewed`, `Reviewed`, `Escalated`, `ReviewFailed`
- `ReviewMode`: `None`, `FirstPass`, `EscalatedPass`

## `violations.json` Shape

Suggested top-level structure:

- run metadata
- deterministic summary
- raw match count
- clustered finding count
- list of clustered findings

Purpose:

- Persist the deterministic stage
- Allow later review/report steps to run independently

## `final_report.json` Shape

Suggested top-level structure:

- run metadata
- input metadata
- deterministic summary
- review summary
- final findings
- review errors

Purpose:

- Serve as the canonical machine-readable output

## JSON Schema Generation

Optional:

- Use `schemars` later if you want machine-generated JSON Schemas for documentation or validation.

Do not block implementation on this.

## Schema Stability Rule

Once implementation begins, do not casually change schema fields.

If a field changes:

- update fixture JSON
- update tests
- update prompt construction
- update reporting

This file should be treated as the contract source of truth.
