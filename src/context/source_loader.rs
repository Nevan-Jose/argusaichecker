use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Load the source text for a given file path.
///
/// Resolution order:
///   1. The path as written (absolute, or relative to the current working directory).
///   2. `source_dir` / `file` (the full relative path joined under the source root).
///   3. `source_dir` / basename only (for cases where the anchor path has an extra prefix).
///
/// Returns the file contents and the resolved path that succeeded.
pub fn load_file(file: &str, source_dir: &Path) -> Result<(String, PathBuf)> {
    let candidates = resolve_candidates(file, source_dir);

    for candidate in &candidates {
        if candidate.exists() {
            let content = std::fs::read_to_string(candidate)
                .with_context(|| format!("Failed to read source file: {:?}", candidate))?;
            return Ok((content, candidate.clone()));
        }
    }

    anyhow::bail!(
        "Source file not found for '{}'. Tried: {}",
        file,
        candidates
            .iter()
            .map(|p| format!("{:?}", p))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn resolve_candidates(file: &str, source_dir: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // 1. As written (absolute or relative to cwd)
    candidates.push(PathBuf::from(file));

    // 2. source_dir / file
    candidates.push(source_dir.join(file));

    // 3. source_dir / basename only
    if let Some(name) = Path::new(file).file_name() {
        candidates.push(source_dir.join(name));
    }

    candidates
}

/// Infer a language name from the file extension.
pub fn language_from_path(path: &str) -> &'static str {
    let p = Path::new(path);
    match p.extension().and_then(|e| e.to_str()) {
        Some("py")               => "python",
        Some("js")               => "javascript",
        Some("ts") | Some("tsx") => "typescript",
        Some("go")               => "go",
        Some("rs")               => "rust",
        Some("java")             => "java",
        Some("rb")               => "ruby",
        Some("php")              => "php",
        Some("cs")               => "csharp",
        Some("cpp") | Some("cc") | Some("cxx") => "cpp",
        Some("c")                => "c",
        Some("sh") | Some("bash") => "shell",
        _                        => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_path() {
        assert_eq!(language_from_path("foo/bar.py"), "python");
        assert_eq!(language_from_path("foo/bar.go"), "go");
        assert_eq!(language_from_path("foo/bar.js"), "javascript");
        assert_eq!(language_from_path("foo/bar.xyz"), "unknown");
    }
}
