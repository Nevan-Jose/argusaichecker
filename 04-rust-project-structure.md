# Rust Project Structure

## Purpose

This file defines the Rust crate layout, dependency choices, and module boundaries for the MVP.

The point is to remove ambiguity before implementation starts.

## Crate Strategy

Use a single binary crate for the MVP.

Why:

- Easier to navigate
- Faster to scaffold
- Less ceremony than a workspace
- Enough structure can still be achieved through modules

If the project grows later, internal modules can be extracted into crates.

## Core Dependencies

### Required Early

- `serde`
- `serde_json`
- `clap`
- `anyhow`
- `thiserror`
- `regex`
- `globset`
- `walkdir`
- `tracing`
- `tracing-subscriber`

### Add Later

- `tokio`
- `reqwest`
- `async-trait`
- `sha2` or `blake3`
- `schemars`

Only add async/network crates once the mock provider path already works.

## Directory Layout

```text
src/
  main.rs
  cli.rs
  app.rs
  config.rs
  schemas/
    mod.rs
    layer1.rs
    policy.rs
    violations.rs
    review.rs
    report.rs
  ingest/
    mod.rs
    loader.rs
    normalize.rs
  policy/
    mod.rs
    compiler.rs
    matcher.rs
    dedupe.rs
    severity.rs
  ranking/
    mod.rs
    ranker.rs
  context/
    mod.rs
    source_loader.rs
    snippet.rs
    packer.rs
  review/
    mod.rs
    provider.rs
    mock.rs
    orchestrator.rs
    prompt.rs
    validator.rs
    cache.rs
  reporting/
    mod.rs
    json_report.rs
    markdown.rs
    gitlab.rs
  utils/
    mod.rs
    fs.rs
    hashing.rs
    ids.rs
samples/
tests/
```

## Module Ownership

### `cli`

Owns:

- Argument parsing
- CLI help text
- Conversion from args to config

Must not own:

- Business logic
- File parsing
- Scanning logic

### `app`

Owns:

- Top-level orchestration of the full pipeline
- Stage ordering
- Error propagation

Must not own:

- Detailed matching logic
- Prompt construction details

### `schemas`

Owns:

- All typed input/output data structures
- Shared enums and report models

Must not own:

- Runtime logic
- Matching logic

### `ingest`

Owns:

- Reading input files
- Deserializing Layer 1 output
- Normalizing records into internal anchors

Must not own:

- Policy matching
- Ranking

### `policy`

Owns:

- Compiling rules
- Deterministic matching
- Dedupe/clustering
- Severity summary generation

Must not own:

- LLM interaction
- Source snippet extraction

### `ranking`

Owns:

- Deciding which findings are worth AI review

Must not own:

- Prompt logic
- Final report rendering

### `context`

Owns:

- Source file loading
- Snippet extraction
- Compact context packet construction

Must not own:

- Provider calls
- Rule matching

### `review`

Owns:

- Provider abstraction
- Mock provider
- Prompt templates
- Response validation
- Escalation logic
- Cache behavior

Must not own:

- Deterministic finding generation
- Markdown rendering

### `reporting`

Owns:

- `violations.json` output
- `final_report.json`
- Markdown audit report
- Optional GitLab export

Must not own:

- Matching and review logic

## First Successful Run

The first successful run does not need AI.

It only needs to:

- Load sample inputs
- Validate schemas
- Produce deterministic violations
- Write `violations.json`

Everything else comes after that baseline works.

## Structural Rules

- Keep `main.rs` minimal
- Keep all stage logic in small modules
- Keep schema types separate from compiled runtime types where needed
- Keep async code isolated to the review provider layer
- Avoid global state

## Testing Layout

Use:

- unit tests inside modules where practical
- integration tests in `tests/`
- fixture-driven tests using `samples/`

## Growth Path

If the MVP works and the project grows later, the most likely extraction path is:

- `schemas` into a shared library
- `policy` into a deterministic engine crate
- `review` into an AI-review crate

Do not do that refactor before the MVP exists.
