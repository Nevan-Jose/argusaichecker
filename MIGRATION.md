# Policy Subsystem Migration Guide

This document describes how the policy subsystem changed from the legacy
single-file format to the new per-rule directory format, and how to migrate
existing rule definitions.

---

## What Changed

| Aspect | Legacy | New |
|---|---|---|
| Storage | One `policy.json` with a `"rules"` array | One JSON file per rule |
| Path format | Flat, no path constraint | `<framework>/<category>/<code>.json` |
| Rule identity | `rule_id` string (free-form) | `id = framework.category.code`, enforced from path |
| CLI argument | `--policy <file>` | `--policy-dir <directory>` |
| Config field | `policy_path: PathBuf` | `policy_dir: PathBuf` |
| Matching | Regex `token_patterns` + glob `path_patterns` | Recursive condition tree (`match` field) |
| Scope filtering | Glob `path_patterns` + language filter | `scope` object with `node_kinds`, `module_patterns`, `endpoint_protocols`, `require_tags` |
| Category | Enum (`secrets`, `crypto`, `injection`, …) | Free-form string from path segment |
| Severity | `info`, `low`, `medium`, `high`, `critical` | `low`, `medium`, `high`, `critical` |
| Finding anchor | Implicit (first token match) | `finding.primary_anchor` (required fact path) |

---

## CLI Usage

```sh
# Legacy
argusaichecker --tokens tokens.json --policy policy.json --source-dir src/

# New
argusaichecker --tokens tokens.json --policy-dir policies/ --source-dir src/
```

---

## Directory Layout

```
policies/
  soc2/
    auth/
      cc2.json          → id: soc2.auth.cc2
    secrets/
      cc6.json          → id: soc2.secrets.cc6
  owasp/
    injection/
      a03.json          → id: owasp.injection.a03
    execution/
      a09.json          → id: owasp.execution.a09
    deserialization/
      a08.json          → id: owasp.deserialization.a08
    insecure_design/
      a04.json          → id: owasp.insecure_design.a04
```

Every `*.json` file under the policy root is loaded. Files that fail
schema parsing or path-to-identity validation cause a hard error at startup.

---

## Rule Format

Full schema: `policies/policy-rule.schema.json`

### Required fields

```json
{
  "id": "owasp.injection.a03",
  "title": "...",
  "description": "...",
  "policy": { "framework": "owasp", "category": "injection", "code": "a03" },
  "severity": "high",
  "languages": ["*"],
  "scope": {},
  "match": { "fact": "token", "op": "matches", "value": "SELECT" },
  "finding": {
    "message": "...",
    "primary_anchor": "token"
  }
}
```

### `match` — condition tree

The `match` field is a recursive condition tree:

| Shape | Meaning |
|---|---|
| `{ "fact": "...", "op": "...", "value": ... }` | Leaf predicate |
| `{ "all": [ <cond>, ... ] }` | All children must be true |
| `{ "any": [ <cond>, ... ] }` | At least one child must be true |
| `{ "not": <cond> }` | Child must be false |

Supported operators: `equals`, `not_equals`, `in`, `not_in`, `contains`,
`not_contains`, `exists`, `not_exists`, `matches` (regex), `taint_reaches`
(stub — always false in the current engine).

### Resolvable facts

Facts are resolved from the `Layer1Anchor`:

| Fact path | Source |
|---|---|
| `token` | `anchor.token` |
| `language` | `anchor.language` |
| `file` | `anchor.file` |
| `token_kind` | `anchor.token_kind` |
| `normalized_kind` | `anchor.normalized_kind` |
| `layer1_rule_id` | `anchor.layer1_rule_id` |
| `layer1_confidence` | `anchor.layer1_confidence` |
| `ast_metadata.<key>` | `anchor.ast_metadata[key]` |

### Language wildcard

```json
"languages": ["*"]
```

Matches all languages. Use specific values (`"go"`, `"python"`, etc.) only
when a rule depends on language-specific extraction behavior.

---

## Migrating Legacy Rules

### `token_patterns` → `match` with `matches` operator

**Legacy:**
```json
"token_patterns": ["AKIA[0-9A-Z]{16}", "(?i)ghp_[0-9A-Za-z]{36}"]
```

**New** (using `any` to combine patterns):
```json
"match": {
  "any": [
    { "fact": "token", "op": "matches", "value": "AKIA[0-9A-Z]{16}" },
    { "fact": "token", "op": "matches", "value": "(?i)ghp_[0-9A-Za-z]{36}" }
  ]
}
```

If you have both `token_patterns` (content match) and `path_patterns`
(file scope), use `all`:

```json
"scope": {
  "module_patterns": ["src/**/*.py"]
},
"match": {
  "any": [
    { "fact": "token", "op": "matches", "value": "AKIA[0-9A-Z]{16}" }
  ]
}
```

### `path_patterns` → `scope.module_patterns`

**Legacy:**
```json
"path_patterns": ["src/**/*.py", "src/**/*.js"]
```

**New:**
```json
"scope": {
  "module_patterns": ["src/**/*.py", "src/**/*.js"]
}
```

### `category` (enum) → path segment

The legacy `category` enum (`secrets`, `crypto`, `injection`, etc.) is now
derived from the second path segment of the rule file:

```
soc2/secrets/cc6.json   → category = "secrets"
owasp/injection/a03.json → category = "injection"
```

No explicit `category` field is needed; it is inferred from the path.

### `rule_id` → `id`

**Legacy:** `"rule_id": "POL-SECRET-001"`
**New:** `"id": "soc2.secrets.cc6"` (must match path)

### `policy_hints` → `references`

**Legacy:**
```json
"policy_hints": ["Move secrets to a secrets manager.", "Rotate credentials immediately."]
```

**New:**
```json
"references": [
  "Move secrets to a secrets manager.",
  "Rotate credentials immediately."
]
```

References populate the "What To Fix" section of audit reports.

### `review_required` and `dedupe_radius_lines`

These fields are no longer in the rule JSON. The engine derives them:

- `review_required` is `true` when `severity` is `high` or `critical`.
- Deduplication uses a fixed radius of 5 lines.

### `adjacent_bug_checks`

No direct equivalent in the new format. Remove these fields.
Adjacent-risk commentary can be placed in `description` or `references`.

### `metadata_filters`

No direct equivalent. If you were using `metadata_filters` to inspect
`ast_metadata` fields, translate them to predicates in the `match` tree:

**Legacy:**
```json
"metadata_filters": { "parent": "assignment" }
```

**New:**
```json
"match": {
  "fact": "ast_metadata.parent",
  "op": "equals",
  "value": "assignment"
}
```

---

## Validation Enforced at Load Time

1. Every rule file must be at exactly `<framework>/<category>/<code>.json`.
2. `id` must equal `<framework>.<category>.<code>`.
3. `policy.framework`, `policy.category`, `policy.code` must match path segments.
4. `finding.primary_anchor` must be non-empty.
5. `languages` must not be empty (`["*"]` is valid for all-language rules).

Any violation causes the engine to exit with a clear error rather than
silently skip the rule.
