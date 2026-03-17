# Fixtures And Demo Corpus

## Purpose

This file defines the sample vulnerable codebase and JSON fixtures used to build and test the pipeline.

Everything in the implementation should be driven by this corpus first.

## Why This Matters

- Gives you a stable test bench
- Prevents you from building against abstract ideas only
- Lets you verify deterministic matching before adding AI
- Gives you repeatable demo inputs
- Makes prompt testing much cheaper

## Sample Source Tree

Recommended layout:

```text
samples/
  src/
    python/
      hardcoded_secret.py
      weak_crypto.py
      sql_injection.py
    js/
      dangerous_exec.js
      insecure_auth.js
    go/
      insecure_deser.go
  tokens.json
  policy.json
```

You do not need all of these if time is tight. Start with 4-6 files total.

## Required Sample Cases

### Hardcoded Secret

Goal:

- Show a clear secret-like literal in code
- Demonstrate deterministic detection
- Let AI confirm whether the secret is real or test-only

Expected behavior:

- Deterministic layer flags it
- AI may confirm or downgrade if clearly fake/test data

### Weak Crypto

Goal:

- Show `md5`, `sha1`, or similarly weak primitive usage
- Include some surrounding code so AI can judge whether the code is in a sensitive path

Expected behavior:

- Deterministic layer flags weak crypto
- AI may adjust severity based on actual use

### SQL Interpolation

Goal:

- Include a query built through string interpolation or concatenation
- Include nearby user input or request parameter use

Expected behavior:

- Deterministic layer flags interpolation
- AI should identify whether untrusted input reaches the query

### Dangerous Shell Execution

Goal:

- Include `exec`, shell invocation, or equivalent
- Include user-controllable input nearby

Expected behavior:

- Deterministic layer flags a dangerous sink
- AI checks whether input is controlled and whether escaping exists

### Insecure Auth / Policy Bypass

Goal:

- Include a disabled verification flag or bypass condition
- Include nearby auth or policy-related control flow

Expected behavior:

- Deterministic layer flags suspicious auth logic
- AI checks whether this is a real auth risk or a benign test toggle

### Unsafe Deserialization

Goal:

- Include obvious deserialization or dynamic loading usage
- Include a source of data nearby if possible

Expected behavior:

- Deterministic layer flags risky pattern
- AI looks for adjacent exploitability clues

## Design Rules For Sample Files

- Keep files short
- Make anchor lines easy to inspect
- Include enough surrounding context for a small snippet to be useful
- Avoid overcomplicated code
- Keep samples realistic, not toy nonsense

## `tokens.json` Expectations

This file should include one normalized Layer 1 anchor entry for each sample issue.

Each record should include:

- File path
- Language
- Line and column
- Token text or token kind
- Optional AST metadata
- Optional Layer 1 rule label or confidence

## `policy.json` Expectations

This file should include one rule per issue family at minimum.

Each rule should define:

- Rule id
- Title
- Description
- Severity
- Category
- Applicable languages
- Token patterns
- Path patterns if needed
- Dedupe radius
- Whether AI review is required
- Policy hints
- Adjacent bug checks

## Ground Truth Notes

For each sample file, add a short section in this document containing:

- Intended issue type
- Whether it should definitely be flagged
- Whether it should go to AI review
- What nearby hidden issue the AI might surface
- What a reasonable remediation would be

This becomes your human reference when testing the system.

## Suggested Ground Truth Template

```md
### samples/src/python/sql_injection.py
- Issue family:
- Anchor line:
- Should deterministic layer flag it?:
- Should AI review it?:
- Possible adjacent hidden bug:
- Expected remediation:
```

## Success Criteria For The Demo Corpus

The demo corpus is ready when:

- Every sample has a clear purpose
- `tokens.json` points to the right locations
- `policy.json` contains matching rules
- You can manually explain what each expected finding should be
- At least one case is ambiguous enough for the AI layer to add value

## Important Constraint

Do not keep adding more samples once the first 4-6 are working.

A small, high-quality corpus is much more useful for building the MVP than a large, messy one.
