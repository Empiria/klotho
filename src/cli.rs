use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "klotho")]
#[command(about = "Run AI agents in isolated containers with persistent Zellij sessions")]
#[command(version)]
pub struct Cli {
    /// Container runtime to use (auto-detected if not specified)
    #[arg(long, global = true, default_value = "auto")]
    pub runtime: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new session or attach to existing one
    Start {
        /// Agent to use (default: "claude")
        #[arg(short, long, default_value = "claude")]
        agent: String,

        /// Session name (default: "default")
        #[arg(short, long, default_value = "default")]
        name: String,

        /// Project paths to mount
        paths: Vec<String>,
    },

    /// Stop a running session
    Stop {
        /// Session name (default: "default")
        #[arg(default_value = "default")]
        name: String,
    },

    /// Start a stopped session and reattach
    Restart {
        /// Session name (default: "default")
        #[arg(default_value = "default")]
        name: String,
    },

    /// List all sessions with status
    Ls,

    /// Remove a stopped session
    Rm {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,

        /// Session name (default: "default")
        #[arg(default_value = "default")]
        name: String,
    },

    /// Build agent container image
    Build {
        /// Build all agents
        #[arg(long)]
        all: bool,

        /// Agent name(s) to build
        agents: Vec<String>,
    },

    /// Rebuild agent container image (no cache)
    Rebuild {
        /// Rebuild all agents
        #[arg(long)]
        all: bool,

        /// Agent name(s) to rebuild
        agents: Vec<String>,
    },
}
