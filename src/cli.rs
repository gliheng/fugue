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

    /// Deploy a function, Nuxt.js, or React Router application
    Deploy {
        /// Function name
        name: String,

        /// Path to JavaScript file or project directory (Nuxt.js, React Router)
        path: String,

        /// Skip build for framework projects (use existing build directory)
        #[arg(long)]
        skip_build: bool,

        /// Environment variables for framework projects (KEY=VALUE format)
        #[arg(short, long)]
        env: Vec<String>,
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
