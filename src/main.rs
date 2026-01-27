use anyhow::Result;
use clap::Parser;
use klotho::cli::{Cli, Commands};
use klotho::commands;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Runtime override is available via cli.runtime
    // "auto" means auto-detect, otherwise use specified runtime
    let runtime_override = if cli.runtime == "auto" {
        None
    } else {
        Some(cli.runtime.as_str())
    };

    match cli.command {
        Commands::Start { agent, name, linked_dirs, paths } => {
            commands::start::run(agent, name, linked_dirs, paths, runtime_override)?;
            Ok(())
        }
        Commands::Stop { name } => {
            commands::stop::run(name, runtime_override)?;
            Ok(())
        }
        Commands::Restart { name } => {
            commands::restart::run(name, runtime_override)?;
            Ok(())
        }
        Commands::Ls => {
            commands::ls::run(runtime_override)?;
            Ok(())
        }
        Commands::Rm { force, name } => {
            commands::rm::run(name, force, runtime_override)?;
            Ok(())
        }
        Commands::Build { all, agents } => {
            commands::build::run(all, agents, false, runtime_override)?;
            Ok(())
        }
        Commands::Rebuild { all, agents } => {
            commands::build::run(all, agents, true, runtime_override)?;
            Ok(())
        }
    }
}
