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
        /// Agent to use (interactive selection if not specified)
        #[arg(short, long)]
        agent: Option<String>,

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

        /// Install additional packages (format: manager:package or manager:package=version)
        #[arg(long = "install")]
        install_packages: Vec<String>,

        /// Agent name(s) to build
        agents: Vec<String>,
    },

    /// Rebuild agent container image (no cache)
    Rebuild {
        /// Rebuild all agents
        #[arg(long)]
        all: bool,

        /// Install additional packages (format: manager:package or manager:package=version)
        #[arg(long = "install")]
        install_packages: Vec<String>,

        /// Agent name(s) to rebuild
        agents: Vec<String>,
    },

    /// Initialize a .klotho.toml configuration file
    Init {
        /// Initialize global config (~/.config/klotho/config.toml) instead of project config
        #[arg(long)]
        global: bool,
    },

    /// Manage klotho configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Manage mobile access via hapi
    Mobile {
        #[command(subcommand)]
        command: MobileCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show merged configuration and validate settings
    Check,
    /// Migrate credentials from host config to klotho config
    Migrate {
        /// Write to global config instead of project config
        #[arg(long)]
        global: bool,
    },
}

#[derive(Subcommand)]
pub enum MobileCommands {
    /// Start the hapi mobile hub sidecar
    Start,
    /// Stop the hapi mobile hub sidecar
    Stop,
    /// Show mobile hub status, URL, QR code, and connected devices
    Status,
    /// Unpair a connected mobile device
    Revoke,
}
