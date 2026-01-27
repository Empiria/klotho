use anyhow::Result;
use clap::Parser;
use klotho::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Runtime override is available via cli.runtime
    // "auto" means auto-detect, otherwise use specified runtime
    let _runtime_override = if cli.runtime == "auto" {
        None
    } else {
        Some(cli.runtime.clone())
    };

    match cli.command {
        Commands::Start { agent, name, paths } => {
            println!("start: agent={}, name={}, paths={:?}", agent, name, paths);
            todo!("Implement start command")
        }
        Commands::Stop { name } => {
            println!("stop: name={}", name);
            todo!("Implement stop command")
        }
        Commands::Restart { name } => {
            println!("restart: name={}", name);
            todo!("Implement restart command")
        }
        Commands::Ls => {
            println!("ls");
            todo!("Implement ls command")
        }
        Commands::Rm { force, name } => {
            println!("rm: force={}, name={}", force, name);
            todo!("Implement rm command")
        }
        Commands::Build { all, agents } => {
            println!("build: all={}, agents={:?}", all, agents);
            todo!("Implement build command")
        }
        Commands::Rebuild { all, agents } => {
            println!("rebuild: all={}, agents={:?}", all, agents);
            todo!("Implement rebuild command")
        }
    }
}
