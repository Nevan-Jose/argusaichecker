use crate::config::Config;
use anyhow::Result;
use tracing::{error, info};

pub fn run(config: Config) -> Result<()> {
    info!("Starting Argus pipeline");

    if let Err(err) = std::fs::create_dir_all(&config.output_dir) {
        error!(
            "Failed to create output directory '{}': {}",
            config.output_dir.display(),
            err
        );
        return Err(err.into());
    }

    // Stage 1+2: Load and normalize anchors from Layer 1 output.
    let anchors = match crate::ingest::load_and_normalize(&config.tokens_path) {
        Ok(anchors) => anchors,
        Err(err) => {
            error!("Failed to load tokens from {:?}: {}", config.tokens_path, err);
            return Err(err);
        }
    };
    info!("Loaded {} anchors", anchors.len());

    // Stage 3+4: Load policy rules from directory and compile.
    let policy_rules = match crate::ingest::load_policy(&config.policy_dir) {
        Ok(rules) => rules,
        Err(err) => {
            error!("Failed to load policy from {:?}: {}", config.policy_dir, err);
            return Err(err);
        }
    };

    let compiled_rules = match crate::policy::compile_rules(policy_rules) {
        Ok(compiled_rules) => compiled_rules,
        Err(err) => {
            error!("Failed to compile policy rules: {}", err);
            return Err(err);
        }
    };
    let raw_matches = crate::policy::match_anchors(&anchors, &compiled_rules);
    info!("Raw matches: {}", raw_matches.len());

    // Stage 5: Dedupe and cluster nearby hits.
    let clusters = crate::policy::dedupe_and_cluster(raw_matches, &compiled_rules);
    info!("Clustered findings: {}", clusters.len());

    // Stage 6: Severity summary.
    let summary = crate::policy::build_summary(&clusters);

    // Stage 7: Write violations.json (deterministic — no AI required).
    let violations_path = config.output_dir.join("violations.json");
    if let Err(err) = crate::reporting::write_violations(&violations_path, &summary, &clusters) {
        error!(
            "Failed to write violations output to {:?}: {}",
            violations_path,
            err
        );
        return Err(err);
    }
    info!("Written violations.json");

    // Stage 8: Rank findings for review.
    let candidates = crate::ranking::rank(&clusters);
    info!(
        "{} findings selected for review",
        candidates.len()
    );

    // Stage 9: Run review — mock or live based on flag / GEMINI_API_KEY.
    let reviews = crate::review::run_review(
        config.mock_review,
        &candidates,
        &clusters,
        &config.source_dir,
    );
    info!("{} findings reviewed", reviews.len());

    // Stage 10: Write final outputs with full provenance.
    let report_path = config.output_dir.join("final_report.json");
    let md_path = config.output_dir.join("audit_report.md");

    if let Err(err) = crate::reporting::write_final_report(
        &report_path,
        &summary,
        &clusters,
        &reviews,
        &config.tokens_path.to_string_lossy(),
        &config.policy_dir.to_string_lossy(),
        &config.source_dir.to_string_lossy(),
    ) {
        error!("Failed to write final JSON report to {:?}: {}", report_path, err);
        return Err(err);
    }

    if let Err(err) = crate::reporting::write_markdown(&md_path, &summary, &clusters, &reviews) {
        error!("Failed to write markdown report to {:?}: {}", md_path, err);
        return Err(err);
    }
    info!("Reports written to {:?}", config.output_dir);

    println!("Argus scan complete.");
    println!("  violations.json   -> {:?}", violations_path);
    println!("  final_report.json -> {:?}", report_path);
    println!("  audit_report.md   -> {:?}", md_path);

    Ok(())
}
