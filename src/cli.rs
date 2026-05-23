use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fugue")]
#[command(about = "Serverless platform using Rust and workerd", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the Fugue platform (foreground)
    Start {
        /// PostgreSQL database URL
        #[arg(long)]
        db_url: Option<String>,

        /// Platform HTTP port
        #[arg(long, default_value = "3000")]
        port: u16,
    },

    /// Stop the Fugue platform
    Stop,

    /// Check platform status
    Status,

    /// Create a new app
    Create {
        /// App name
        name: String,

        /// Framework: worker, nuxtjs, react-router
        #[arg(short, long, default_value = "worker")]
        framework: String,

        /// App description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Deploy an app (upload source + build + deploy)
    Deploy {
        /// App name or ID
        name: String,

        /// Path to project directory or JavaScript file
        path: String,

        /// Skip build for framework projects
        #[arg(long)]
        skip_build: bool,

        /// Environment variables (KEY=VALUE format)
        #[arg(short, long)]
        env: Vec<String>,
    },

    /// List all apps
    List,

    /// Show app info
    Info {
        /// App name or ID
        name: String,
    },

    /// Delete an app
    Delete {
        /// App name or ID
        name: String,
    },

    /// Get the URL of a deployed app
    Url {
        /// App name or ID
        name: String,
    },

    /// View app logs
    Logs {
        /// App name or ID
        name: String,
    },

    /// Start a stopped app
    StartApp {
        /// App name or ID
        name: String,
    },

    /// Stop a running app
    StopApp {
        /// App name or ID
        name: String,
    },
}
