use crate::cli::Args;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub tokens_path: PathBuf,
    pub policy_path: PathBuf,
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub mock_review: bool,
}

impl Config {
    pub fn from_args(args: Args) -> Result<Self> {
        Ok(Config {
            tokens_path: args.tokens,
            policy_path: args.policy,
            source_dir: args.source_dir,
            output_dir: args.output_dir,
            mock_review: args.mock_review,
        })
    }
}
