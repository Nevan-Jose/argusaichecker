use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "argusaichecker", about = "Layer 2 AI security/compliance review agent")]
pub struct Args {
    #[arg(long, help = "Path to tokens.json from Layer 1")]
    pub tokens: PathBuf,

    #[arg(long, help = "Path to policy rules directory (contains <framework>/<category>/<code>.json files)")]
    pub policy_dir: PathBuf,

    #[arg(long, help = "Source directory for context extraction")]
    pub source_dir: PathBuf,

    #[arg(long, default_value = "out", help = "Output directory")]
    pub output_dir: PathBuf,

    #[arg(long, default_value = "false", help = "Use mock AI provider")]
    pub mock_review: bool,
}

pub fn parse() -> Args {
    Args::parse()
}
