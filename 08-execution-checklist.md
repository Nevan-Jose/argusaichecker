# Execution Checklist

## Purpose

This file is the working build order.

It turns the architecture into a chronological task list you can execute directly.

## Phase 1: Scope And Research

- [ ] Write the product definition in `01-product-scope.md`
- [ ] Write the non-goals in `01-product-scope.md`
- [ ] Research Semgrep, Gitleaks, Bandit, GuardDog, and GitLab formats
- [ ] Record notes in `02-reference-research.md`
- [ ] Decide which structural ideas to adopt
- [ ] Decide the first 4-6 finding categories
- [ ] Decide the first languages to demo

Done means:

- You can explain the product in under one minute
- You know what similar tools you are learning from

## Phase 2: Fixtures And Contracts

- [ ] Create the sample vulnerable source files described in `03-fixtures-and-demo-corpus.md`
- [ ] Create `samples/tokens.json`
- [ ] Create `samples/policy.json`
- [ ] Write expected outcomes for each sample
- [ ] Define all JSON contracts in `05-data-contracts-and-schemas.md`
- [ ] Freeze the first version of the schemas

Done means:

- You have a fixed demo corpus
- You can describe expected findings before writing the engine

## Phase 3: Rust Scaffolding

- [ ] Initialize the Rust binary crate
- [ ] Add core dependencies
- [ ] Create the `src/` module layout from `04-rust-project-structure.md`
- [ ] Add empty modules
- [ ] Make `cargo check` pass
- [ ] Add `main.rs`, `cli.rs`, and `app.rs` scaffolding

Done means:

- The repo has a stable structure
- The crate compiles with stubs in place

## Phase 4: Schemas And Parsing

- [ ] Implement schema structs in `src/schemas/`
- [ ] Add enums for severity, category, and review status
- [ ] Add serde derives
- [ ] Add tests that parse `samples/tokens.json`
- [ ] Add tests that parse `samples/policy.json`
- [ ] Add tests for serializing report structures

Done means:

- Input and output shapes are real code
- Fixtures parse cleanly

## Phase 5: Ingest And Normalize

- [ ] Implement file loading in `src/ingest/loader.rs`
- [ ] Implement anchor normalization in `src/ingest/normalize.rs`
- [ ] Add tests for normalization behavior
- [ ] Confirm normalized anchors are stable and predictable

Done means:

- Layer 1 input is converted into a consistent internal form

## Phase 6: Deterministic Policy Engine

- [ ] Implement compiled policy rules in `src/policy/compiler.rs`
- [ ] Implement deterministic matching in `src/policy/matcher.rs`
- [ ] Implement nearby dedupe in `src/policy/dedupe.rs`
- [ ] Implement severity summaries in `src/policy/severity.rs`
- [ ] Write `violations.json`
- [ ] Add tests for matching, dedupe, and summaries

Done means:

- The non-AI engine produces useful deterministic findings

## Phase 7: Validate The Non-AI Baseline

- [ ] Run the tool on the sample corpus
- [ ] Read `violations.json` manually
- [ ] Confirm the expected findings appear
- [ ] Confirm noise is acceptable
- [ ] Fix rules or normalization if needed

Done means:

- The deterministic layer feels like a useful product already

## Phase 8: Ranking And Context

- [ ] Implement the ranker in `src/ranking/ranker.rs`
- [ ] Implement source loading in `src/context/source_loader.rs`
- [ ] Implement snippet extraction in `src/context/snippet.rs`
- [ ] Implement compact context packets in `src/context/packer.rs`
- [ ] Add tests for ranking and snippet packing

Done means:

- You can produce compact review candidates without any model calls

## Phase 9: Mock AI Review Path

- [ ] Implement the provider abstraction in `src/review/provider.rs`
- [ ] Implement the mock provider in `src/review/mock.rs`
- [ ] Implement the triage result schema
- [ ] Implement strict response validation
- [ ] Implement review orchestration
- [ ] Implement merge logic into final findings
- [ ] Add end-to-end tests using the mock provider

Done means:

- The full pipeline works offline with structured AI-like output

## Phase 10: Reporting

- [ ] Implement `final_report.json`
- [ ] Implement markdown report output
- [ ] Include deterministic summary and review summary
- [ ] Include remediation and adjacent hidden faults
- [ ] Add report snapshot or fixture tests

Done means:

- The tool emits demo-ready reports

## Phase 11: Real Provider Integration

- [ ] Add async/network dependencies
- [ ] Implement the real provider
- [ ] Add prompt templates
- [ ] Add strict JSON handling
- [ ] Add retry/repair logic for invalid responses
- [ ] Add environment-based API key loading
- [ ] Manually test with a few findings only

Done means:

- Real model review works for a small controlled set of findings

## Phase 12: Escalation And Caching

- [ ] Add second-pass escalation rules
- [ ] Add wider-snippet or enclosing-function context
- [ ] Add cache keys based on snippet hash and prompt version
- [ ] Reuse cached results on repeated runs
- [ ] Test escalation and cache behavior

Done means:

- Token usage is controlled and reruns are cheaper

## Phase 13: GitLab Export

- [ ] Decide the exact GitLab report format to support
- [ ] Implement export mapping
- [ ] Add tests for the export schema
- [ ] Confirm the native report remains the system of record

Done means:

- The tool can plug into a GitLab-oriented workflow

## Phase 14: Optional Remediation Preview

- [ ] Add remediation text generation if not already present
- [ ] Add fix-preview structures
- [ ] Restrict actual code changes to very safe cases only
- [ ] Keep implementation opt-in

Done means:

- The system can suggest or preview fixes without undermining trust

## Phase 15: Hardening And Demo Prep

- [ ] Clean dead code
- [ ] Run `cargo fmt`
- [ ] Run `cargo clippy`
- [ ] Run `cargo test`
- [ ] Verify a clean end-to-end demo command
- [ ] Verify mock mode works offline
- [ ] Prepare one example of false-positive reduction
- [ ] Prepare one example of nearby hidden-bug discovery
- [ ] Prepare one example of escalation

Done means:

- The project is ready to show

## Prompting Workflow For Claude/Codex

Use this order when asking for help:

1. Scaffold modules only
2. Define schemas
3. Implement CLI and config
4. Implement loading and normalization
5. Implement policy compilation
6. Implement deterministic matching
7. Implement dedupe and summaries
8. Implement ranking and context packing
9. Implement provider abstraction and mock provider
10. Implement review validation and orchestration
11. Implement final reporting
12. Implement the real provider
13. Add escalation and caching

## Daily Operating Rule

Never ask the model to do architecture changes and feature additions in the same prompt.

Finish one thin slice, run it, then move on.
