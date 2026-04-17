use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fugue")]
#[command(about = "Serverless platform POC using Rust and workerd", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the daemon server
    Start,

    /// Stop the daemon server
    Stop,

    /// Check daemon status
    Status,

    /// Deploy a function or Next.js application
    Deploy {
        /// Function name
        name: String,

        /// Path to JavaScript file or Next.js project directory
        path: String,

        /// Skip build for Next.js projects (use existing .next directory)
        #[arg(long)]
        skip_build: bool,

        /// Environment variables for Next.js projects (KEY=VALUE format)
        #[arg(short, long)]
        env: Vec<String>,
    },

    /// Deploy a Next.js application (deprecated: use 'deploy' instead)
    #[command(hide = true)]
    DeployNextjs {
        /// Function name
        name: String,

        /// Path to Next.js project directory
        directory: String,

        /// Skip build and use existing .next directory
        #[arg(long)]
        skip_build: bool,

        /// Environment variables (KEY=VALUE format)
        #[arg(short, long)]
        env: Vec<String>,
    },

    /// Rebuild a deployed Next.js application
    Rebuild {
        /// Function name
        name: String,
    },

    /// Get the URL of a deployed function
    Url {
        /// Function name
        name: String,
    },

    /// Invoke a function
    Invoke {
        /// Function name
        name: String,

        /// JSON data to pass to function
        #[arg(short, long)]
        data: Option<String>,
    },

    /// List all deployed functions
    List,

    /// Delete a function
    Delete {
        /// Function name
        name: String,
    },

    /// View function logs
    Logs {
        /// Function name
        name: String,
    },
}
