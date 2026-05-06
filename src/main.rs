mod cli;
mod client;
mod commands;
mod config;
mod daemon;
mod error;
mod nuxtjs;
mod registry;
mod runtime;
mod validation;

use clap::Parser;
use cli::{Cli, Commands};
use error::Result;

#[tokio::main]
async fn main() {
    // Check if running as daemon
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "__daemon" {
        if let Err(e) = daemon::start_daemon().await {
            eprintln!("Daemon error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let cli = Cli::parse();

    let result: Result<()> = match cli.command {
        Commands::Start => commands::start_command().await,
        Commands::Stop => commands::stop_command().await,
        Commands::Status => commands::status_command().await,
        Commands::Deploy { name, path, skip_build, env } => {
            commands::deploy_command(name, path, skip_build, env).await
        }
        Commands::Url { name } => commands::url_command(name).await,
        Commands::Invoke { name, data } => commands::invoke_command(name, data).await,
        Commands::List => commands::list_command().await,
        Commands::Delete { name } => commands::delete_command(name).await,
        Commands::Logs { name } => commands::logs_command(name).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
