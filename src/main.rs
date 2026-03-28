use clap::{Parser, Subcommand};
use mcp_prism::{AppConfig, error::AppResult, server};
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Parser)]
#[command(name = "mcp-prism")]
#[command(about = "Rust MCP aggregation pool with proxy-aware routing.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Serve,
    Stdio,
    PrintConfig,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = AppConfig::load()?;
    init_tracing(config.log_format);

    match Cli::parse().command.unwrap_or(Command::Serve) {
        Command::Serve => server::serve(config).await,
        Command::Stdio => server::run_stdio(config).await,
        Command::PrintConfig => {
            println!(
                "{}",
                serde_json::to_string_pretty(&config.redacted_snapshot())?
            );
            Ok(())
        }
    }
}

fn init_tracing(log_format: mcp_prism::config::LogFormat) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    match log_format {
        mcp_prism::config::LogFormat::Json => {
            fmt().with_env_filter(filter).json().init();
        }
        mcp_prism::config::LogFormat::Pretty => {
            fmt().with_env_filter(filter).compact().init();
        }
    }
}
