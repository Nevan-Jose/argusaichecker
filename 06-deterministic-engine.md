# Deterministic Engine

## Purpose

This file describes the non-AI scanning pipeline.

This layer must work well on its own before any model provider is added.

## Why The Deterministic Layer Comes First

- It provides useful output even without AI
- It creates the anchors the AI will investigate
- It reduces noise before spending tokens
- It gives you a testable baseline
- It makes the overall system more trustworthy

## Stage 1: Input Loading

Responsibilities:

- Read `tokens.json`
- Read `policy.json`
- Validate file existence
- Parse JSON into typed structs
- Return readable errors on malformed input

Important rule:

- Input loading should not contain matching or ranking logic

## Stage 2: Anchor Normalization

Responsibilities:

- Convert raw Layer 1 records into a single normalized anchor type
- Normalize paths
- Normalize language names
- Normalize token kinds
- Preserve line and column information
- Preserve optional AST metadata without depending on it

Why:

- The rest of the pipeline should not depend on raw Layer 1 quirks

## Stage 3: Policy Compilation

Responsibilities:

- Compile regexes once
- Compile path-glob matchers once
- Normalize rule language filters
- Validate broken or contradictory rules early

Why:

- Improves runtime simplicity
- Surfaces policy errors at startup

## Stage 4: Deterministic Matching

Responsibilities:

- Test normalized anchors against compiled rules
- Emit raw matches when token patterns and filters align
- Attach rule metadata to each match
- Attach a short reason string explaining the match

Important:

- Keep rule logic explainable
- Avoid overly clever scoring here
- Favor stable, predictable behavior

## Stage 5: Nearby Dedupe And Clustering

Responsibilities:

- Group raw matches by file and rule
- Sort matches by location
- Merge nearby matches within the configured line radius
- Preserve original evidence inside each cluster

Why:

- Prevents repeated nearby hits from flooding reports
- Produces cleaner inputs for the AI layer

## Stage 6: Severity Summary

Responsibilities:

- Count raw matches
- Count clustered findings
- Count findings by severity
- Count findings by category
- Count findings by file

Why:

- Supports reporting
- Gives fast scan visibility

## `violations.json`

This output should include:

- run metadata
- deterministic summary
- raw match counts
- clustered findings
- enough evidence to inspect results manually

This file should be treated as a complete, usable output of the deterministic layer.

## Implementation Constraints

- No model calls
- No snippet packing
- No prompt logic
- No review-time severity changes

The deterministic stage should remain independently understandable.

## Basic Matching Strategy

For the MVP, deterministic matching should focus on:

- token text patterns
- token kind patterns
- optional path filters
- optional lightweight metadata filters

Do not overengineer a rich rule language in v1.

## Success Criteria

The deterministic layer is ready when:

- It can load the sample fixtures
- It emits expected findings for the demo corpus
- Dedupe visibly reduces noise
- `violations.json` is readable and useful
- The AI layer can consume its output without needing schema changes

## Recommended Test Coverage

- valid token parsing
- valid policy parsing
- rule compilation success/failure
- expected deterministic hits
- non-matching anchors staying clean
- nearby dedupe behavior
- severity summary counts
- `violations.json` emission

## Common Failure Modes To Watch

- Rules matching too broadly
- Path normalization bugs
- Broken regexes surfacing late
- Duplicate findings overwhelming the report
- Summary counts not matching actual outputs

## Rule Of Thumb

Do not move to the AI layer until this stage feels like a useful product by itself.
