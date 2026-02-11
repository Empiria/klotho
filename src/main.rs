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
        Commands::Start { agent, name, paths } => {
            commands::start::run(agent, name, paths, runtime_override)?;
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
        Commands::Build { all, install_packages, agents } => {
            commands::build::run(all, agents, install_packages, false, runtime_override)?;
            Ok(())
        }
        Commands::Rebuild { all, install_packages, agents } => {
            commands::build::run(all, agents, install_packages, true, runtime_override)?;
            Ok(())
        }
        Commands::Init { global } => {
            commands::init::run(global)?;
            Ok(())
        }
        Commands::Mobile { command } => {
            match command {
                klotho::cli::MobileCommands::Start => {
                    commands::mobile::start::run(runtime_override)?;
                }
                klotho::cli::MobileCommands::Stop => {
                    commands::mobile::stop::run(runtime_override)?;
                }
                klotho::cli::MobileCommands::Status => {
                    commands::mobile::status::run(runtime_override)?;
                }
                klotho::cli::MobileCommands::Revoke => {
                    commands::mobile::revoke::run(runtime_override)?;
                }
            }
            Ok(())
        }
    }
}
