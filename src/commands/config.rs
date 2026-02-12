use anyhow::{Context, Result};
use dialoguer::Confirm;
use owo_colors::OwoColorize;
use std::path::PathBuf;

use crate::config::{get_config_home, load_global_config, merge_configs};
use crate::project_config::{load_project_config, resolve_env_vars};

/// Run `klotho config check` - show merged config and validate
pub fn run_check() -> Result<()> {
    let global_path = get_config_home().join("config.toml");
    let project_path = std::env::current_dir()?.join(".klotho.toml");

    let global = load_global_config()?;
    let project = load_project_config(&std::env::current_dir()?)?;
    let resolved = merge_configs(&global, &project);

    println!("{}", "Configuration Sources".bold());
    println!("  Global:  {}", format_path_status(&global_path));
    println!("  Project: {}", format_path_status(&project_path));
    println!();

    println!("{}", "Resolved Configuration".bold());
    println!(
        "  runtime:           {}",
        resolved.runtime.as_deref().unwrap_or("auto")
    );
    println!(
        "  default_agent:     {}",
        resolved.default_agent.as_deref().unwrap_or("(interactive)")
    );
    println!("  mount_host_config: {}", resolved.mount_host_config);
    println!();

    // Show volumes
    if !resolved.volumes.is_empty() {
        println!("{}", "Volumes".bold());
        for vol in &resolved.volumes {
            let (src, dest, readonly) = vol.resolve();
            let ro_flag = if readonly { " (ro)" } else { "" };
            println!("  {} → {}{}", src, dest, ro_flag);
        }
        println!();
    }

    // Show agent credentials (masked)
    if !resolved.agents.is_empty() {
        println!("{}", "Agent Credentials".bold());
        for (name, creds) in &resolved.agents {
            let key_status = match &creds.api_key {
                Some(k) if k.contains("${") => {
                    // Try to resolve env var to check if it's set
                    match resolve_env_vars(k) {
                        Ok(_) => format!("{} (env var resolved)", "set".green()),
                        Err(_) => format!("{}", "env var NOT SET".red()),
                    }
                }
                Some(_) => format!("{}", "set".green()),
                None => format!("{}", "not configured".yellow()),
            };
            println!("  {}.api_key: {}", name, key_status);
        }
        println!();
    }

    // Show MCP servers
    if !resolved.mcp.servers.is_empty()
        || !resolved.mcp.claude.is_empty()
        || !resolved.mcp.opencode.is_empty()
    {
        println!("{}", "MCP Servers".bold());
        for name in resolved.mcp.servers.keys() {
            println!("  {} (shared)", name);
        }
        for name in resolved.mcp.claude.keys() {
            println!("  {} (claude)", name);
        }
        for name in resolved.mcp.opencode.keys() {
            println!("  {} (opencode)", name);
        }
        println!();
    }

    println!("{} Configuration valid", "✓".green());

    Ok(())
}

fn format_path_status(path: &PathBuf) -> String {
    if path.exists() {
        format!("{} {}", path.display(), "(found)".green())
    } else {
        format!("{} {}", path.display(), "(not found)".dimmed())
    }
}

/// Run `klotho config migrate` - extract credentials from host config
pub fn run_migrate(global: bool) -> Result<()> {
    println!("{}", "Credential Migration".bold());
    println!();

    // Detect existing host credentials
    let home = std::env::var("HOME").context("HOME not set")?;
    let claude_dir = PathBuf::from(&home).join(".claude");
    let opencode_dir = PathBuf::from(&home).join(".config/opencode");

    // Check Claude credentials
    if claude_dir.exists() {
        println!("  {} ~/.claude directory found", "→".cyan());
        println!("    Claude uses OAuth tokens stored in this directory.");
        println!("    For API key usage, set ANTHROPIC_API_KEY environment variable");
        println!("    and reference it in config: api_key = \"${{ANTHROPIC_API_KEY}}\"");
        println!();
    }

    // Check OpenCode credentials
    if opencode_dir.exists() {
        println!("  {} ~/.config/opencode directory found", "→".cyan());
        println!("    OpenCode config may contain provider API keys.");
        println!("    Set appropriate env vars (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.)");
        println!("    and reference them in config: api_key = \"${{ANTHROPIC_API_KEY}}\"");
        println!();
    }

    // Check for existing env vars
    let mut env_suggestions: Vec<(&str, &str)> = Vec::new();
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        env_suggestions.push(("claude", "ANTHROPIC_API_KEY"));
    }
    if std::env::var("OPENAI_API_KEY").is_ok() {
        env_suggestions.push(("opencode", "OPENAI_API_KEY"));
    }

    if env_suggestions.is_empty() && !claude_dir.exists() && !opencode_dir.exists() {
        println!("No existing credentials found to migrate.");
        println!();
        println!("To configure credentials, add to your config file:");
        println!();
        println!("  [agents.claude]");
        println!("  api_key = \"${{ANTHROPIC_API_KEY}}\"");
        println!();
        return Ok(());
    }

    // Generate config snippet
    if !env_suggestions.is_empty() {
        println!("{}", "Suggested Configuration".bold());
        println!();
        println!("Add the following to your config file:");
        println!();
        for (agent, env_var) in &env_suggestions {
            println!("  [agents.{}]", agent);
            println!("  api_key = \"${{{}}}\"", env_var);
            println!();
        }

        // Determine target file
        let target_path = if global {
            get_config_home().join("config.toml")
        } else {
            std::env::current_dir()?.join(".klotho.toml")
        };

        // Ask user if they want to append
        let should_write = Confirm::new()
            .with_prompt(format!("Append to {}?", target_path.display()))
            .default(false)
            .interact()
            .unwrap_or_else(|_| {
                // Non-interactive mode - print instructions instead
                println!("Run in an interactive terminal to automatically append, or copy the config above manually.");
                false
            });

        if should_write {
            let mut content = String::new();
            content.push_str("\n# Agent credentials (migrated by klotho config migrate)\n");
            for (agent, env_var) in &env_suggestions {
                content.push_str(&format!("[agents.{}]\n", agent));
                content.push_str(&format!("api_key = \"${{{}}}\"\n\n", env_var));
            }

            // Ensure parent directory exists for global config
            if global {
                std::fs::create_dir_all(get_config_home())?;
            }

            // Append to file (create if doesn't exist)
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&target_path)
                .context(format!("Failed to open {}", target_path.display()))?;
            file.write_all(content.as_bytes())?;

            println!(
                "{} Credentials written to {}",
                "✓".green(),
                target_path.display()
            );

            // Offer to disable host mounts
            if claude_dir.exists() || opencode_dir.exists() {
                println!();
                println!("To disable host config mounting (use only klotho config), add:");
                println!("  mount_host_config = false");
            }
        }
    }

    Ok(())
}
