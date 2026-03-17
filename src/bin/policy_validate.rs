use clap::Parser;
use regex::Regex;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ─── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "policy_validate", about = "Validate Argus policy rule files")]
struct Args {
    #[arg(long)]
    policy_dir: PathBuf,

    #[arg(long)]
    schema_path: PathBuf,
}

// ─── Output types (match the required JSON schema exactly) ───────────────────

#[derive(Serialize)]
struct Report {
    overall: Overall,
    summary: Summary,
    violations: Vec<Violation>,
}

#[derive(Serialize)]
struct Overall {
    status: String,
    total_files: usize,
    schema_pass: usize,
    format_pass: usize,
    trace_pass: usize,
    schema_fail: usize,
    format_fail: usize,
    trace_fail: usize,
}

#[derive(Serialize)]
struct Summary {
    by_framework: BTreeMap<String, usize>,
    mapped_controls: BTreeMap<String, Vec<String>>,
    trace_issues: TraceIssues,
}

#[derive(Serialize)]
struct TraceIssues {
    missing_reference_count: usize,
    non_official_reference_count: usize,
}

#[derive(Serialize)]
struct Violation {
    file_path: String,
    framework: String,
    #[serde(rename = "type")]
    kind: String,
    severity: String,
    issue: String,
    field: String,
    required_fix: String,
}

// ─── Per-file result (internal) ───────────────────────────────────────────────

struct FileResult {
    framework: String,
    schema_violations: Vec<Violation>,
    format_violations: Vec<Violation>,
    trace_violations: Vec<Violation>,
    /// controls found in mappings, keyed by framework name
    mapped_controls: HashMap<String, Vec<String>>,
    /// number of references that lack an official domain for this file's framework
    non_official_ref_count: usize,
    /// true when references array is absent or empty
    missing_references: bool,
}

// ─── Constants ────────────────────────────────────────────────────────────────

const VALID_SEVERITIES: &[&str] = &["low", "medium", "high", "critical"];
const VALID_LANGUAGES: &[&str] = &[
    "*", "go", "java", "python", "rust", "typescript", "javascript",
];
const VALID_OPS: &[&str] = &[
    "equals",
    "not_equals",
    "in",
    "not_in",
    "contains",
    "not_contains",
    "exists",
    "not_exists",
    "matches",
    "taint_reaches",
];
const ALLOWED_TOP: &[&str] = &[
    "id",
    "title",
    "description",
    "policy",
    "severity",
    "languages",
    "scope",
    "match",
    "finding",
    "mappings",
    "references",
    "tags",
    "defaults",
];
const ALLOWED_SCOPE: &[&str] = &[
    "node_kinds",
    "module_patterns",
    "endpoint_protocols",
    "require_tags",
];
const ALLOWED_FINDING: &[&str] = &["message", "primary_anchor", "related"];
const ALLOWED_MAPPINGS: &[&str] = &["soc2", "gdpr", "owasp"];
const VALID_FRAMEWORKS: &[&str] = &["soc2", "gdpr", "owasp"];

fn official_domains(framework: &str) -> &'static [&'static str] {
    match framework {
        "soc2" => &["aicpa.org", "csrc.nist.gov"],
        "owasp" => &["owasp.org"],
        "gdpr" => &["eur-lex.europa.eu", "edpb.europa.eu"],
        _ => &[],
    }
}

fn has_official_domain(s: &str, framework: &str) -> bool {
    let lower = s.to_lowercase();
    official_domains(framework).iter().any(|d| lower.contains(d))
}

/// Compiled-once regexes for control ID validation.
struct ControlRegexes {
    soc2: Regex,
    owasp: Regex,
    gdpr: Regex,
}

impl ControlRegexes {
    fn new() -> Self {
        Self {
            // CCxx, CC6.1, CC6.1_2
            soc2: Regex::new(r"^CC\d+(\.\d+)?(_\d+)?$").unwrap(),
            // A01..A10 with optional :YYYY qualifier  e.g. "A07:2021"
            owasp: Regex::new(r"^A(?:0[1-9]|10)(:\d{4})?$").unwrap(),
            // Art.25, Art.25(1), Art.32(1)(a)
            gdpr: Regex::new(r"^Art\.\d+(\(\d+\)(\([a-z]\))?)?$").unwrap(),
        }
    }

    fn validate(&self, control: &str, framework: &str) -> bool {
        match framework {
            "soc2" => self.soc2.is_match(control),
            "owasp" => self.owasp.is_match(control),
            "gdpr" => self.gdpr.is_match(control),
            _ => false,
        }
    }
}

// ─── Match condition validator ────────────────────────────────────────────────

fn validate_condition(v: &Value, errs: &mut Vec<String>, path: &str) {
    let obj = match v.as_object() {
        Some(o) => o,
        None => {
            errs.push(format!("{path}: condition must be an object"));
            return;
        }
    };

    if obj.contains_key("fact") || obj.contains_key("op") {
        validate_predicate(obj, errs, path);
    } else if let Some(children) = obj.get("all") {
        check_no_extra_keys(obj, &["all"], errs, path);
        validate_condition_array(children, errs, &format!("{path}.all"));
    } else if let Some(children) = obj.get("any") {
        check_no_extra_keys(obj, &["any"], errs, path);
        validate_condition_array(children, errs, &format!("{path}.any"));
    } else if let Some(inner) = obj.get("not") {
        check_no_extra_keys(obj, &["not"], errs, path);
        validate_condition(inner, errs, &format!("{path}.not"));
    } else {
        errs.push(format!(
            "{path}: unrecognised condition — expected predicate(fact+op), all, any, or not"
        ));
    }
}

fn validate_predicate(obj: &Map<String, Value>, errs: &mut Vec<String>, path: &str) {
    for k in obj.keys() {
        if !matches!(k.as_str(), "fact" | "op" | "value") {
            errs.push(format!("{path}: unknown predicate field '{k}'"));
        }
    }
    match obj.get("fact").and_then(Value::as_str) {
        Some(f) if !f.is_empty() => {}
        _ => errs.push(format!("{path}.fact: must be a non-empty string")),
    }
    match obj.get("op").and_then(Value::as_str) {
        Some(op) if VALID_OPS.contains(&op) => {}
        Some(op) => errs.push(format!("{path}.op: invalid op '{op}'")),
        None => errs.push(format!("{path}.op: required")),
    }
}

fn validate_condition_array(v: &Value, errs: &mut Vec<String>, path: &str) {
    match v.as_array() {
        Some(arr) if arr.is_empty() => {
            errs.push(format!("{path}: must be non-empty array"));
        }
        Some(arr) => {
            for (i, item) in arr.iter().enumerate() {
                validate_condition(item, errs, &format!("{path}[{i}]"));
            }
        }
        None => errs.push(format!("{path}: must be array")),
    }
}

fn check_no_extra_keys(
    obj: &Map<String, Value>,
    allowed: &[&str],
    errs: &mut Vec<String>,
    path: &str,
) {
    for k in obj.keys() {
        if !allowed.contains(&k.as_str()) {
            errs.push(format!("{path}: unexpected key '{k}'"));
        }
    }
}

// ─── Core validation ──────────────────────────────────────────────────────────

fn validate_file(path: &Path, policies_dir: &Path, ctrl_re: &ControlRegexes) -> FileResult {
    let file_path = path.to_string_lossy().to_string();

    // Derive framework / category / code from path
    let rel = path.strip_prefix(policies_dir).unwrap_or(path);
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    let (framework, category, code) = if parts.len() == 3 {
        let code = parts[2].trim_end_matches(".json").to_string();
        (parts[0].clone(), parts[1].clone(), code)
    } else {
        ("unknown".to_string(), "unknown".to_string(), "unknown".to_string())
    };

    let mut sv: Vec<Violation> = Vec::new(); // schema violations
    let mut fv: Vec<Violation> = Vec::new(); // format violations
    let mut tv: Vec<Violation> = Vec::new(); // trace / crossref violations
    let mut mapped_controls: HashMap<String, Vec<String>> = HashMap::new();

    macro_rules! sviol {
        ($sev:expr, $issue:expr, $field:expr, $fix:expr) => {
            sv.push(Violation {
                file_path: file_path.clone(),
                framework: framework.clone(),
                kind: "schema".to_string(),
                severity: $sev.to_string(),
                issue: $issue.to_string(),
                field: $field.to_string(),
                required_fix: $fix.to_string(),
            })
        };
    }
    macro_rules! fviol {
        ($sev:expr, $issue:expr, $field:expr, $fix:expr) => {
            fv.push(Violation {
                file_path: file_path.clone(),
                framework: framework.clone(),
                kind: "format".to_string(),
                severity: $sev.to_string(),
                issue: $issue.to_string(),
                field: $field.to_string(),
                required_fix: $fix.to_string(),
            })
        };
    }
    macro_rules! tviol {
        ($type:expr, $sev:expr, $issue:expr, $field:expr, $fix:expr) => {
            tv.push(Violation {
                file_path: file_path.clone(),
                framework: framework.clone(),
                kind: $type.to_string(),
                severity: $sev.to_string(),
                issue: $issue.to_string(),
                field: $field.to_string(),
                required_fix: $fix.to_string(),
            })
        };
    }

    // ── Read & parse ──────────────────────────────────────────────────────────
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            sviol!("critical", format!("Cannot read file: {e}"), "", "Fix file IO error");
            return FileResult {
                framework,
                schema_violations: sv,
                format_violations: fv,
                trace_violations: tv,
                mapped_controls,
                non_official_ref_count: 0,
                missing_references: true,
            };
        }
    };

    let root: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            sviol!("critical", format!("Invalid JSON: {e}"), "", "Fix JSON syntax");
            return FileResult {
                framework,
                schema_violations: sv,
                format_violations: fv,
                trace_violations: tv,
                mapped_controls,
                non_official_ref_count: 0,
                missing_references: true,
            };
        }
    };

    let top = match root.as_object() {
        Some(o) => o,
        None => {
            sviol!("critical", "Root value is not a JSON object", "", "Wrap content in a JSON object");
            return FileResult {
                framework,
                schema_violations: sv,
                format_violations: fv,
                trace_violations: tv,
                mapped_controls,
                non_official_ref_count: 0,
                missing_references: true,
            };
        }
    };

    // ── Schema: required fields ───────────────────────────────────────────────
    for req in ["id", "title", "description", "policy", "severity", "languages", "scope", "match", "finding"] {
        if !top.contains_key(req) {
            sviol!(
                "critical",
                format!("Missing required field '{req}'"),
                req,
                format!("Add required field '{req}'")
            );
        }
    }

    // ── Schema: unknown top-level fields ──────────────────────────────────────
    for key in top.keys() {
        if !ALLOWED_TOP.contains(&key.as_str()) {
            sviol!(
                "high",
                format!("Unknown top-level field '{key}'"),
                key,
                format!("Remove field '{key}'")
            );
        }
    }

    // ── Schema: scope ─────────────────────────────────────────────────────────
    match top.get("scope") {
        Some(s) if s.is_object() => {
            for k in s.as_object().unwrap().keys() {
                if !ALLOWED_SCOPE.contains(&k.as_str()) {
                    sviol!(
                        "high",
                        format!("Unknown field 'scope.{k}'"),
                        format!("scope.{k}"),
                        format!("Remove 'scope.{k}'")
                    );
                }
            }
        }
        Some(_) => sviol!("critical", "Field 'scope' must be an object", "scope", "Change scope to an object"),
        None => {} // already caught by required-fields check
    }

    // ── Schema: finding ───────────────────────────────────────────────────────
    match top.get("finding") {
        Some(f) if f.is_object() => {
            let fobj = f.as_object().unwrap();
            for k in fobj.keys() {
                if !ALLOWED_FINDING.contains(&k.as_str()) {
                    sviol!(
                        "high",
                        format!("Unknown field 'finding.{k}'"),
                        format!("finding.{k}"),
                        format!("Remove 'finding.{k}'")
                    );
                }
            }
            match fobj.get("message").and_then(Value::as_str) {
                Some(m) if !m.is_empty() => {}
                Some(_) | None => sviol!(
                    "critical",
                    "finding.message is required and must be non-empty",
                    "finding.message",
                    "Set a non-empty finding.message"
                ),
            }
            match fobj.get("primary_anchor").and_then(Value::as_str) {
                Some(a) if !a.is_empty() => {}
                Some(_) | None => sviol!(
                    "critical",
                    "finding.primary_anchor is required and must be non-empty",
                    "finding.primary_anchor",
                    "Set a non-empty finding.primary_anchor"
                ),
            }
        }
        Some(_) => sviol!("critical", "Field 'finding' must be an object", "finding", "Change finding to an object"),
        None => {} // already caught
    }

    // ── Schema: mappings unknown keys ─────────────────────────────────────────
    if let Some(m) = top.get("mappings") {
        match m.as_object() {
            Some(mobj) => {
                for k in mobj.keys() {
                    if !ALLOWED_MAPPINGS.contains(&k.as_str()) {
                        sviol!(
                            "high",
                            format!("Unknown key 'mappings.{k}'"),
                            format!("mappings.{k}"),
                            "Use only soc2, gdpr, owasp as mapping keys"
                        );
                    }
                }
            }
            None => sviol!("high", "Field 'mappings' must be an object", "mappings", "Change mappings to an object"),
        }
    }

    // ── Schema: severity ──────────────────────────────────────────────────────
    match top.get("severity").and_then(Value::as_str) {
        Some(s) if VALID_SEVERITIES.contains(&s) => {}
        Some(s) => sviol!(
            "high",
            format!("Invalid severity '{s}'"),
            "severity",
            "Use one of: low, medium, high, critical"
        ),
        None => {} // already caught
    }

    // ── Schema: languages ─────────────────────────────────────────────────────
    match top.get("languages").and_then(Value::as_array) {
        Some(arr) if arr.is_empty() => {
            sviol!("high", "languages must be non-empty", "languages", "Add at least one language");
        }
        Some(arr) => {
            for item in arr {
                match item.as_str() {
                    Some(l) if VALID_LANGUAGES.contains(&l) => {}
                    Some(l) => sviol!(
                        "high",
                        format!("Invalid language '{l}'"),
                        "languages",
                        format!("Use one of: {}", VALID_LANGUAGES.join(", "))
                    ),
                    None => sviol!("medium", "Non-string value in languages", "languages", "All language values must be strings"),
                }
            }
        }
        None => {
            if top.get("languages").is_some() {
                sviol!("high", "languages must be an array", "languages", "Change languages to an array");
            }
        }
    }
    if false && top.get("languages").is_some() && !top["languages"].is_array() {
        sviol!("high", "languages must be an array", "languages", "Change languages to an array");
    }

    // ── Schema: match condition ───────────────────────────────────────────────
    if let Some(match_val) = top.get("match") {
        let mut cond_errs: Vec<String> = Vec::new();
        validate_condition(match_val, &mut cond_errs, "match");
        for e in cond_errs {
            sviol!("high", e, "match", "Fix match condition structure");
        }
    }

    // ── Format: ID pattern ────────────────────────────────────────────────────
    let id_re = Regex::new(r"^[a-z0-9_]+\.[a-z0-9_]+\.[a-z0-9_]+$").unwrap();
    let id = top.get("id").and_then(Value::as_str).unwrap_or("");
    if !id.is_empty() {
        if !id_re.is_match(id) {
            fviol!(
                "critical",
                format!("id '{id}' does not match ^[a-z0-9_]+.[a-z0-9_]+.[a-z0-9_]+$"),
                "id",
                "Fix id to <framework>.<category>.<code>"
            );
        } else {
            // Path identity
            let expected = format!("{framework}.{category}.{code}");
            if id != expected {
                fviol!(
                    "critical",
                    format!("id '{id}' does not match path-derived '{expected}'"),
                    "id",
                    format!("Change id to '{expected}'")
                );
            }
        }
    }

    // ── Format: policy object path identity ───────────────────────────────────
    if let Some(pol) = top.get("policy").and_then(Value::as_object) {
        let pfw = pol.get("framework").and_then(Value::as_str).unwrap_or("");
        let pcat = pol.get("category").and_then(Value::as_str).unwrap_or("");
        let pcode = pol.get("code").and_then(Value::as_str).unwrap_or("");

        if !pfw.is_empty() && !VALID_FRAMEWORKS.contains(&pfw) {
            fviol!(
                "high",
                format!("policy.framework '{pfw}' is not a valid framework"),
                "policy.framework",
                "Use one of: soc2, gdpr, owasp"
            );
        }
        if pfw != framework {
            fviol!(
                "critical",
                format!("policy.framework '{pfw}' != path framework '{framework}'"),
                "policy.framework",
                format!("Change policy.framework to '{framework}'")
            );
        }
        if pcat != category {
            fviol!(
                "critical",
                format!("policy.category '{pcat}' != path category '{category}'"),
                "policy.category",
                format!("Change policy.category to '{category}'")
            );
        }
        if pcode != code {
            fviol!(
                "critical",
                format!("policy.code '{pcode}' != path code '{code}'"),
                "policy.code",
                format!("Change policy.code to '{code}'")
            );
        }

        // Also check unknown/missing policy fields
        for req in ["framework", "category", "code"] {
            if !pol.contains_key(req) {
                sviol!(
                    "critical",
                    format!("policy.{req} is required"),
                    format!("policy.{req}"),
                    format!("Add policy.{req}")
                );
            }
        }
        for k in pol.keys() {
            if !matches!(k.as_str(), "framework" | "category" | "code") {
                sviol!(
                    "high",
                    format!("Unknown field 'policy.{k}'"),
                    format!("policy.{k}"),
                    format!("Remove 'policy.{k}'")
                );
            }
        }
    }

    // ── Trace: mappings control IDs ───────────────────────────────────────────
    let mut has_any_mapping = false;

    if let Some(mobj) = top.get("mappings").and_then(Value::as_object) {
        for fw in ALLOWED_MAPPINGS {
            if let Some(arr) = mobj.get(*fw).and_then(Value::as_array) {
                if arr.is_empty() {
                    tviol!(
                        "trace",
                        "low",
                        format!("mappings.{fw} is present but empty"),
                        format!("mappings.{fw}"),
                        format!("Add at least one {fw} control or remove the key")
                    );
                }
                for item in arr {
                    if let Some(ctrl) = item.as_str() {
                        has_any_mapping = true;
                        mapped_controls
                            .entry(fw.to_string())
                            .or_default()
                            .push(ctrl.to_string());

                        if !ctrl_re.validate(ctrl, fw) {
                            tviol!(
                                "trace",
                                "medium",
                                format!("Control ID '{ctrl}' in mappings.{fw} has unexpected format"),
                                format!("mappings.{fw}"),
                                format!("Use realistic {fw} control ID (e.g. CC6.1 / A03 / Art.25(1))")
                            );
                        }
                    } else {
                        tviol!(
                            "trace",
                            "medium",
                            format!("mappings.{fw} contains non-string entry"),
                            format!("mappings.{fw}"),
                            "All control IDs must be strings"
                        );
                    }
                }
            }
        }
    } else {
        tviol!(
            "trace",
            "high",
            "No mappings object present",
            "mappings",
            "Add a mappings object with soc2/gdpr/owasp controls"
        );
    }

    // ── Trace: references ─────────────────────────────────────────────────────
    let refs: Vec<&str> = top
        .get("references")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    let missing_references = refs.is_empty();

    if !has_any_mapping && missing_references {
        tviol!(
            "trace",
            "high",
            "No mappings and no references — no traceability evidence",
            "mappings/references",
            "Add mappings and/or references"
        );
    } else if missing_references {
        tviol!(
            "trace",
            "medium",
            "No references provided",
            "references",
            "Add at least one reference URL to an official source"
        );
    }

    // ── Crossref: official domain check per reference string ─────────────────
    let mut non_official_ref_count = 0usize;
    for r in &refs {
        if !has_official_domain(r, &framework) {
            non_official_ref_count += 1;
            let snippet = if r.len() > 100 { &r[..100] } else { r };
            tviol!(
                "crossref",
                "low",
                format!("Reference does not contain an official {framework} source domain: \"{snippet}\""),
                "references",
                format!(
                    "Add a reference from an official source: {}",
                    official_domains(&framework).join(", ")
                )
            );
        }
    }

    FileResult {
        framework,
        schema_violations: sv,
        format_violations: fv,
        trace_violations: tv,
        mapped_controls,
        non_official_ref_count,
        missing_references,
    }
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    if !args.policy_dir.exists() {
        eprintln!("policy_dir does not exist: {}", args.policy_dir.display());
        std::process::exit(1);
    }
    if !args.schema_path.exists() {
        eprintln!("schema_path does not exist: {}", args.schema_path.display());
        std::process::exit(1);
    }

    let schema_path_canon = args
        .schema_path
        .canonicalize()
        .unwrap_or_else(|_| args.schema_path.clone());

    let ctrl_re = ControlRegexes::new();

    // Collect rule files: depth-3 JSON under policies_dir, skip schema
    let mut policy_files: Vec<PathBuf> = WalkDir::new(&args.policy_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .filter(|e| {
            e.path()
                .canonicalize()
                .map(|p| p != schema_path_canon)
                .unwrap_or(true)
        })
        .filter(|e| {
            e.path()
                .strip_prefix(&args.policy_dir)
                .map(|rel| rel.components().count() == 3)
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    policy_files.sort();

    // ── Validate all files ────────────────────────────────────────────────────
    let mut all_violations: Vec<Violation> = Vec::new();
    let mut by_framework: BTreeMap<String, usize> = BTreeMap::new();
    let mut all_controls: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut total_missing_refs = 0usize;
    let mut total_non_official = 0usize;

    let mut schema_fail_files: HashSet<String> = HashSet::new();
    let mut format_fail_files: HashSet<String> = HashSet::new();
    let mut trace_fail_files: HashSet<String> = HashSet::new();

    for path in &policy_files {
        let result = validate_file(path, &args.policy_dir, &ctrl_re);
        let fp = path.to_string_lossy().to_string();

        *by_framework.entry(result.framework.clone()).or_insert(0) += 1;

        for (fw, controls) in result.mapped_controls {
            let set = all_controls.entry(fw).or_default();
            for c in controls {
                set.insert(c);
            }
        }

        if result.missing_references {
            total_missing_refs += 1;
        }
        total_non_official += result.non_official_ref_count;

        if !result.schema_violations.is_empty() {
            schema_fail_files.insert(fp.clone());
        }
        if !result.format_violations.is_empty() {
            format_fail_files.insert(fp.clone());
        }
        if !result.trace_violations.is_empty() {
            trace_fail_files.insert(fp.clone());
        }

        all_violations.extend(result.schema_violations);
        all_violations.extend(result.format_violations);
        all_violations.extend(result.trace_violations);
    }

    // ── Build report ──────────────────────────────────────────────────────────
    let total = policy_files.len();
    let schema_fail = schema_fail_files.len();
    let format_fail = format_fail_files.len();
    let trace_fail = trace_fail_files.len();

    let mut mapped_controls_out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for fw in ["soc2", "owasp", "gdpr"] {
        let mut v: Vec<String> = all_controls
            .get(fw)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect();
        v.sort();
        mapped_controls_out.insert(fw.to_string(), v);
    }

    let status = if all_violations.is_empty() { "PASS" } else { "FAIL" };

    let report = Report {
        overall: Overall {
            status: status.to_string(),
            total_files: total,
            schema_pass: total - schema_fail,
            format_pass: total - format_fail,
            trace_pass: total - trace_fail,
            schema_fail,
            format_fail,
            trace_fail,
        },
        summary: Summary {
            by_framework,
            mapped_controls: mapped_controls_out,
            trace_issues: TraceIssues {
                missing_reference_count: total_missing_refs,
                non_official_reference_count: total_non_official,
            },
        },
        violations: all_violations,
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
