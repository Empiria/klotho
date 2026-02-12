use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::process::Command;

use crate::config::{load_global_config, merge_configs};
use crate::container::{container_status, detect_runtime, find_container, ContainerStatus};
use crate::project_config::{load_project_config, SkillConfig};

pub fn run(session: Option<String>, runtime_override: Option<&str>) -> Result<()> {
    // Load and merge configs
    let global_config = load_global_config()?;
    let project_config = load_project_config(&std::env::current_dir()?)?;
    let resolved = merge_configs(&global_config, &project_config);

    if let Some(session_name) = session {
        // Show skills for a specific session
        show_session_skills(&session_name, runtime_override)?;
    } else {
        // Show configured skills (merged global + project)
        show_configured_skills(&resolved.skills)?;
    }

    Ok(())
}

fn show_configured_skills(skills: &HashMap<String, SkillConfig>) -> Result<()> {
    if skills.is_empty() {
        println!("No skills configured.");
        println!();
        println!("Add skills to .klotho.toml:");
        println!("  [skills.gsd]");
        println!("  install = \"npm install -g get-shit-done-cc\"");
        println!("  setup = \"npx get-shit-done-cc --claude --global\"");
        return Ok(());
    }

    println!("{}", "Configured Skills".bold());
    println!();

    for (name, skill) in skills {
        println!("  {} {}", "●".cyan(), name.bold());
        println!("    install: {}", skill.install.dimmed());
        if let Some(setup) = &skill.setup {
            println!("    setup:   {}", setup.dimmed());
        }
        if !skill.agents.is_empty() {
            println!("    agents:  {}", skill.agents.join(", ").dimmed());
        }
        println!();
    }

    Ok(())
}

fn show_session_skills(session_name: &str, runtime_override: Option<&str>) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;

    // Find container
    let container = find_container(runtime, session_name)?
        .context(format!("Session '{}' not found", session_name))?;

    // Check if running
    let status = container_status(runtime, &container)?;
    if status != ContainerStatus::Running {
        println!(
            "Session '{}' is not running. Start it first with:",
            session_name
        );
        println!("  klotho start {}", session_name);
        return Ok(());
    }

    // Get KLOTHO_SKILLS env var from container
    let output = Command::new(runtime.as_str())
        .args(["exec", &container, "printenv", "KLOTHO_SKILLS"])
        .output()
        .context("Failed to read skills from session")?;

    if !output.status.success() || output.stdout.is_empty() {
        println!("No skills installed in session '{}'", session_name);
        return Ok(());
    }

    let skills_json = String::from_utf8_lossy(&output.stdout);
    let skills: HashMap<String, SkillConfig> =
        serde_json::from_str(&skills_json).context("Failed to parse skills from session")?;

    println!(
        "{} (session: {})",
        "Installed Skills".bold(),
        session_name.cyan()
    );
    println!();

    for (name, skill) in &skills {
        println!("  {} {}", "✓".green(), name.bold());
        println!("    install: {}", skill.install.dimmed());
        if let Some(setup) = &skill.setup {
            println!("    setup:   {}", setup.dimmed());
        }
        println!();
    }

    Ok(())
}
