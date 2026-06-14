mod api;
mod cli;
mod client;
mod commands;
mod config;
mod db;
mod nuxtjs;
mod process;
mod reactrouter;
mod runtime;
mod templates;
mod validation;
mod vite;
mod worker;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> fugue_common::error::Result<()> {
    match cli.command {
        Commands::Start { db_url, port } => {
            commands::start_platform(db_url.as_deref(), port).await
        }
        Commands::Stop => {
            println!("Stop command: use Ctrl+C to stop the foreground process");
            Ok(())
        }
        Commands::Status => {
            let config = config::PlatformConfig::load()?;
            let client = client::DaemonClient::new_with_port(config.platform.port);
            match client.status().await {
                Ok(status) => {
                    println!("Platform Status:");
                    println!("{}", serde_json::to_string_pretty(&status)?);
                    Ok(())
                }
                Err(_) => {
                    println!("Platform is not running");
                    Ok(())
                }
            }
        }
        Commands::Create {
            name,
            framework,
            description,
        } => {
            let config = config::PlatformConfig::load()?;
            let client = client::DaemonClient::new_with_port(config.platform.port);
            let app = client.create_app(&name, &framework, description.as_deref()).await?;
            println!("App created:");
            println!("  ID: {}", app.id);
            println!("  Name: {}", app.name);
            println!("  Slug: {}", app.slug);
            println!("  Subdomain: {}", app.subdomain);
            println!("  Framework: {}", app.framework);
            println!("  Status: {}", app.status);
            println!(
                "  URL: http://{}.{}:{}",
                app.subdomain, config.platform.domain, config.platform.port
            );
            Ok(())
        }
        Commands::Deploy {
            name,
            path,
            skip_build: _,
            env: _,
        } => {
            let config = config::PlatformConfig::load()?;
            let client = client::DaemonClient::new_with_port(config.platform.port);
            let app = client.get_app_by_name(&name).await?;

            println!("Deploying app '{}' ({})...", app.name, app.id);

            let path_obj = std::path::Path::new(&path);
            if path_obj.is_file() {
                let result = client.upload_source_file(&app.id, path_obj).await?;
                println!(
                    "Uploaded {} files ({} bytes)",
                    result.file_count, result.total_size
                );
            } else if path_obj.is_dir() {
                let result = client.upload_source_dir(&app.id, path_obj).await?;
                println!(
                    "Uploaded {} files ({} bytes)",
                    result.file_count, result.total_size
                );
            } else {
                return Err(fugue_common::error::FugueError::ValidationError(format!(
                    "Path not found: {}",
                    path
                )));
            }

            let deploy_result = client.deploy(&app.id).await?;
            println!("Build started: {}", deploy_result.build_id);
            println!("Status: {}", deploy_result.status);
            println!();
            println!("Poll status with:");
            println!("  fugue info {}", name);

            Ok(())
        }
        Commands::List => {
            let config = config::PlatformConfig::load()?;
            let client = client::DaemonClient::new_with_port(config.platform.port);
            let apps = client.list_apps().await?;

            if apps.is_empty() {
                println!("No apps deployed");
            } else {
                println!("Apps:");
                println!();
                for app in apps {
                    println!(
                        "  • {} ({}) - {} [{}]",
                        app.name, app.slug, app.framework, app.status
                    );
                    println!(
                        "    URL: http://{}.{}:{}",
                        app.subdomain, config.platform.domain, config.platform.port
                    );
                    println!("    Created: {}", app.created_at);
                    println!();
                }
            }
            Ok(())
        }
        Commands::Info { name } => {
            let config = config::PlatformConfig::load()?;
            let client = client::DaemonClient::new_with_port(config.platform.port);
            let app = client.get_app_by_name(&name).await?;

            println!("App: {}", app.name);
            println!("  ID: {}", app.id);
            println!("  Slug: {}", app.slug);
            println!("  Subdomain: {}", app.subdomain);
            println!("  Framework: {}", app.framework);
            println!("  Status: {}", app.status);
            println!(
                "  URL: http://{}.{}:{}",
                app.subdomain, config.platform.domain, config.platform.port
            );
            if let Some(desc) = &app.description {
                println!("  Description: {}", desc);
            }
            println!("  Created: {}", app.created_at);
            println!("  Updated: {}", app.updated_at);
            Ok(())
        }
        Commands::Delete { name } => {
            let config = config::PlatformConfig::load()?;
            let client = client::DaemonClient::new_with_port(config.platform.port);
            let app = client.get_app_by_name(&name).await?;
            client.delete_app(&app.id).await?;
            println!("App '{}' deleted", name);
            Ok(())
        }
        Commands::Url { name } => {
            let config = config::PlatformConfig::load()?;
            let client = client::DaemonClient::new_with_port(config.platform.port);
            let app = client.get_app_by_name(&name).await?;

            if app.status == "running" {
                println!(
                    "http://{}.{}:{}",
                    app.subdomain, config.platform.domain, config.platform.port
                );
            } else {
                println!("App '{}' is not running (status: {})", name, app.status);
            }
            Ok(())
        }
        Commands::Logs { name } => {
            println!("Logs for '{}':", name);
            println!("Note: Logs not yet implemented");
            Ok(())
        }
        Commands::StartApp { name } => {
            let config = config::PlatformConfig::load()?;
            let client = client::DaemonClient::new_with_port(config.platform.port);
            let app = client.get_app_by_name(&name).await?;
            let result = client.start_app(&app.id).await?;
            println!("App '{}' started", name);
            if let Some(url) = result.get("url").and_then(|v| v.as_str()) {
                println!("URL: {}", url);
            }
            Ok(())
        }
        Commands::StopApp { name } => {
            let config = config::PlatformConfig::load()?;
            let client = client::DaemonClient::new_with_port(config.platform.port);
            let app = client.get_app_by_name(&name).await?;
            client.stop_app(&app.id).await?;
            println!("App '{}' stopped", name);
            Ok(())
        }
    }
}
