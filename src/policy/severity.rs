use crate::schemas::violations::{SeveritySummary, ViolationCluster};
use std::collections::HashMap;

pub fn summarize(clusters: &[ViolationCluster]) -> SeveritySummary {
    let raw_match_count: usize = clusters.iter().map(|c| c.match_count).sum();
    let cluster_count = clusters.len();

    let mut by_severity: HashMap<String, usize> = HashMap::new();
    let mut by_category: HashMap<String, usize> = HashMap::new();
    let mut by_file: HashMap<String, usize> = HashMap::new();

    for c in clusters {
        *by_severity.entry(c.severity.to_string()).or_insert(0) += 1;
        *by_category.entry(c.category.to_string()).or_insert(0) += 1;
        *by_file.entry(c.file.clone()).or_insert(0) += 1;
    }

    SeveritySummary {
        raw_match_count,
        cluster_count,
        by_severity,
        by_category,
        by_file,
    }
}
