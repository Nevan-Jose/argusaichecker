# Reference Research

## Purpose

This file records which existing tools to study for architecture, data modeling, rule organization, and reporting patterns.

The goal is to borrow good ideas without copying the wrong things or creating licensing problems.

## Research Rules

- Prefer using reference repos for architecture ideas, not direct copy-paste.
- Prefer permissive licenses when borrowing implementation patterns.
- Keep notes on exactly what idea is useful from each tool.
- Separate "safe to borrow patterns from" from "study only".

## Primary Reference Tools

### Semgrep

Useful for:

- Rule-driven scanning architecture
- Finding metadata design
- Severity/category modeling
- The idea of combining deterministic analysis with AI post-processing

Watch-outs:

- Core repo licensing is not the same as “copy anything freely without review”.
- The rules repository has its own licensing model.

Use for:

- Structural inspiration
- Rule and finding anatomy
- Report-shape ideas

### Gitleaks

Useful for:

- Secret-detection rule layout
- Config structure
- Simple CLI output/reporting patterns
- Scan ergonomics for a hackathon-friendly tool

Use for:

- Config and reporting inspiration
- High-signal MVP patterns

### Bandit

Useful for:

- Static security scanning structure
- Plugin-style organization
- Rule/check separation
- Simple finding emission flow

Use for:

- Thinking about checker organization
- Mapping findings to stable, explainable outputs

### GuardDog

Useful for:

- Combining static signals and security heuristics
- Clean reporting structure
- Practical scanner composition ideas

Use for:

- High-level architecture ideas
- Risk categorization ideas

### CodeQL

Useful for:

- Finding taxonomy
- Rich metadata on findings
- Query and category organization

Use for:

- Learning how mature tools describe findings
- Understanding report fields worth preserving

Watch-outs:

- Do not overfit the MVP to CodeQL’s complexity.

### TruffleHog

Useful for:

- Secret scanning concepts
- Verification thinking

Watch-outs:

- Strong copyleft licensing means it is better treated as inspiration unless you explicitly want that legal burden.

## GitLab References

Study GitLab docs for:

- SAST artifact shape
- Code Quality artifact shape
- Secure scanner integration approach

The purpose is not to design around GitLab on day one. The purpose is to make later export straightforward.

## What To Borrow

Borrow these kinds of ideas:

- Rule metadata structure
- Severity/category enums
- Report field design
- Scan summary design
- Clear CLI ergonomics
- Stable finding identifiers
- Separation between scanning and reporting

## What Not To Borrow

- Tool-specific complexity that does not help the MVP
- Large plugin ecosystems
- Full query languages
- Whole-repo dataflow engines
- Complex deployment assumptions

## Research Output To Capture

As you study each tool, write down:

- What module layout idea is worth copying
- What schema or field design is useful
- What output/report pattern is useful
- Whether the idea is worth implementing in the MVP
- Whether the source is safe to borrow patterns from

## Minimum Deliverable For This Research File

By the time research is “done enough,” this file should contain:

- A short list of tools studied
- One paragraph on what each is useful for
- A short license note
- A decision on which patterns you will adopt

## Decision Template

Use this format for each repo:

```md
### Tool Name
- Why it matters:
- What to copy structurally:
- What to avoid:
- License note:
- Action for this project:
```

## Final Goal Of This File

This file should help you answer:

- Which tools most closely resemble the architecture you want?
- Which ideas are low-risk and high-value to adopt?
- Which implementation patterns are overkill for the hackathon MVP?
