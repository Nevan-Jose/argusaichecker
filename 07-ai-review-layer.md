# AI Review Layer

## Purpose

This file defines the AI-assisted triage layer that sits on top of deterministic findings.

The role of this layer is narrow:

- confirm likely faults
- reduce false positives
- detect nearby hidden issues
- suggest remediation

It is not meant to replace deterministic scanning.

## Design Principles

- Review only selected findings
- Send minimal context first
- Never send full files by default
- Require structured JSON output
- Escalate only when uncertainty is high or severity is critical
- Preserve deterministic evidence even if AI fails

## Input To The AI Layer

The AI layer should consume:

- deterministic finding clusters
- rule metadata
- small local code snippets
- local path/language metadata
- policy hints

It should not consume:

- full repositories
- whole files by default
- unrelated code

## Stage 1: Ranking

Responsibilities:

- choose which findings deserve AI review
- prioritize findings with the highest value per token spent

Recommended ranking signals:

- critical or high severity
- secrets
- auth logic
- crypto misuse
- dangerous execution
- deserialization
- repeated nearby anchors
- ambiguous patterns likely to generate false positives

The ranker exists to save money and improve signal, not to imitate a full risk engine.

## Stage 2: Context Packing

Responsibilities:

- load only the relevant source file
- cut a small local window around the anchor
- include minimal metadata needed for triage
- include policy hints and adjacent-fault checks

Recommended packet contents:

- finding id
- rule id and title
- category and severity
- file path
- language
- anchor line
- small code window
- deterministic evidence summary
- adjacent bug checklist

## Stage 3: Provider Abstraction

The AI layer should depend on an interface, not directly on a vendor API.

Why:

- easier testing with a mock provider
- easier future swap to Claude/OpenAI/other
- clean separation between orchestration and transport

The provider interface should accept one context packet and return one structured triage result.

## Stage 4: Mock Provider

Build the mock provider before any real model integration.

Responsibilities:

- return stable structured results
- simulate confirmed findings
- simulate false positives
- simulate escalation requests

Why:

- lets the rest of the pipeline be built offline
- avoids wasting credits during core development

## Stage 5: Structured Triage Response

The AI response must be strict JSON.

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

These fields should be validated before they affect the final report.

## Stage 6: First-Pass Review

The first pass should use:

- one finding
- one small snippet
- compact rule context
- strict output schema instructions

The first pass should answer:

- Is this likely real?
- How confident are we?
- Is severity unchanged or adjusted?
- Is there an obvious nearby related fault?
- What is the remediation?
- Is more context required?

## Stage 7: Escalation

Escalation should happen only when:

- the model explicitly asks for more context
- confidence lands in a gray zone
- severity is critical
- the nearby hidden issue needs a wider scope to judge

Escalated context may include:

- a wider snippet window
- the nearest enclosing function or block
- nearby imports
- nearby config flags

Full-file review should remain rare and explicit.

## Stage 8: Validation

Every response must be validated.

Validation should check:

- required fields
- legal enum values
- confidence range
- shape of hidden fault entries
- legal severity adjustment

If invalid:

- attempt one repair or retry
- otherwise mark review as failed and preserve deterministic output

## Stage 9: Caching

Cache review results by:

- rule id
- path
- anchor line
- snippet hash
- policy version
- prompt version

Why:

- reduces repeat cost
- stabilizes reruns

## Stage 10: Merge Behavior

The AI layer should not overwrite deterministic evidence.

It should add:

- confirmation status
- confidence
- severity adjustment
- explanation
- hidden adjacent faults
- remediation
- review provenance

## Hidden Adjacent Bug Discovery

This should stay local and practical.

Good targets:

- user input reaching dangerous sinks nearby
- secrets exposure in the same local block
- weak crypto in a sensitive code path
- insecure auth checks or bypasses
- policy circumvention in the same code region

Bad targets for v1:

- whole-repo speculative reasoning
- long-chain dataflow claims without evidence
- unrelated style or architecture review

## Prompt Design Rules

- keep the system prompt fixed
- keep user prompts short
- include only one finding per prompt
- include strict JSON instructions
- describe the model as an anchor-driven local reviewer
- do not ask for general review

## Success Criteria

The AI layer is successful when:

- it reviews only selected findings
- first-pass packets are compact
- JSON output is valid and stable
- false positives decrease
- at least one nearby hidden issue is surfaced
- failed reviews do not break the pipeline
