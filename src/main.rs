use argusaichecker::cli;
use argusaichecker::config::Config;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = cli::parse();
    let config = Config::from_args(args)?;
    argusaichecker::app::run(config)
}
