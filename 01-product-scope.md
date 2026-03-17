# Product Scope

## Purpose

Build a Rust CLI MVP for a two-layer AI security/compliance review agent for GitLab-style workflows.

The system consumes Layer 1 lexical-analysis output, applies deterministic policy matching, selects only high-value findings for AI review, and emits structured reports with confirmation status, remediation guidance, and nearby hidden-risk observations.

## What The Product Is

- An anchor-driven review system.
- A pipeline that starts from explicit risky anchors found by Layer 1.
- A narrow security/compliance fault reviewer.
- A CLI-first tool that can later export GitLab-friendly artifacts.
- A system designed to minimize token usage by sending only small local code windows to the AI layer.

## What The Product Is Not

- Not a general code review bot.
- Not a whole-repo reasoning engine.
- Not a replacement for Layer 1 lexical analysis.
- Not a full taint-analysis platform in v1.
- Not an always-online system; deterministic scanning must still work without AI.

## MVP Boundary

The MVP should support one complete golden path:

1. Read `tokens.json` or `violations.json` from Layer 1.
2. Read `policy.json`.
3. Load local source snippets from a source directory.
4. Produce deterministic violations.
5. Rank only the most valuable findings for AI review.
6. Pack small local context only.
7. Run structured AI triage.
8. Produce `final_report.json` and a markdown audit report.

## Core Product Principles

- Deterministic first, AI second.
- AI reviews only selected findings, not everything.
- Full files are never sent on first pass.
- Escalation is explicit and rare.
- Every stage has a typed schema.
- The output must remain useful even if the AI provider fails.

## Primary User Story

A developer or security reviewer runs one CLI command against Layer 1 output and a local source tree. The tool filters noise, confirms likely faults, finds nearby related risks, and returns structured remediation-ready results.

## Golden-Path CLI

```bash
cargo run -- \
  --tokens samples/tokens.json \
  --policy samples/policy.json \
  --source-dir samples/src \
  --output-dir out
```

## First Supported Finding Categories

- Hardcoded secrets
- Weak cryptography
- Dangerous shell execution
- SQL interpolation
- Insecure auth/config bypass
- Unsafe deserialization or similar high-risk execution patterns

## First Supported Languages

Pick only a few for the demo. Recommended:

- Python
- JavaScript or TypeScript
- Go or Java

The demo does not need broad language coverage. It only needs enough variety to prove the architecture.

## Input Files

- `tokens.json` or `violations.json` from Layer 1
- `policy.json`
- Optional local source files for context extraction

## Required Outputs

- `violations.json`
- `final_report.json`
- `audit_report.md`

## AI Layer Responsibilities

- Confirm or reject likely findings
- Reduce false positives
- Adjust severity when justified
- Identify nearby hidden faults in local context
- Return remediation guidance
- Request more context only when needed

## Non-Goals For V1

- Full GitLab app integration
- Merge request comments
- Whole-file review by default
- Cross-file dataflow reasoning
- Broad auto-fix support
- Multiple model-provider routing strategies
- UI/dashboard work

## Definition Of MVP Success

The MVP is successful if:

- The CLI runs end to end.
- Deterministic matching works without AI.
- Only selected findings are sent to AI.
- AI returns strict structured JSON.
- The final report is readable and machine-consumable.
- At least one example shows false-positive reduction.
- At least one example shows nearby hidden bug discovery.
