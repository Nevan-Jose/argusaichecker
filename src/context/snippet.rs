/// Extract a snippet of source code centred on `center_line` (1-based).
/// Returns plain lines without line-number annotations.
pub fn extract(source: &str, center_line: u32, radius: u32) -> String {
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return String::new();
    }
    let start = center_line.saturating_sub(radius + 1) as usize;
    let end = ((center_line + radius) as usize).min(lines.len());
    lines[start..end].join("\n")
}

/// Extract a snippet of source code centred on `center_line` (1-based),
/// prepending each line with its line number for readability in prompts
/// and reports.
///
/// Example:
/// ```text
///    6 | def hash_password(password: str) -> str:
///    7 |     return hashlib.md5(password.encode()).hexdigest()
/// ```
pub fn extract_numbered(source: &str, center_line: u32, radius: u32) -> String {
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return String::new();
    }
    let start_idx = center_line.saturating_sub(radius + 1) as usize;
    let end_idx = ((center_line + radius) as usize).min(lines.len());

    lines[start_idx..end_idx]
        .iter()
        .enumerate()
        .map(|(i, line)| format!("{:>4} | {}", start_idx + i + 1, line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "line1\nline2\nline3\nline4\nline5\nline6\nline7";

    #[test]
    fn test_extract_center() {
        let s = extract(SRC, 4, 1);
        assert!(s.contains("line3"), "expected line3, got: {}", s);
        assert!(s.contains("line4"), "expected line4, got: {}", s);
        assert!(s.contains("line5"), "expected line5, got: {}", s);
    }

    #[test]
    fn test_extract_numbered_has_line_numbers() {
        let s = extract_numbered(SRC, 4, 1);
        assert!(s.contains("   3 | line3"), "got: {}", s);
        assert!(s.contains("   4 | line4"), "got: {}", s);
        assert!(s.contains("   5 | line5"), "got: {}", s);
    }

    #[test]
    fn test_extract_at_beginning() {
        let s = extract(SRC, 1, 2);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_extract_beyond_end() {
        let s = extract(SRC, 7, 5);
        assert!(!s.is_empty());
    }
}
