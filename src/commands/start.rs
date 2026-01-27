use anyhow::{bail, Context, Result};
use dialoguer::Select;
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::agent::{self, AgentConfig};
use crate::commands::build;
use crate::config::load_agent_config;
use crate::container::{
    container_status, detect_runtime, find_container, get_image_name, image_exists,
    start_container, ContainerStatus, Runtime,
};
use crate::resources;

pub fn run(
    agent: Option<String>,
    name: String,
    paths: Vec<String>,
    runtime_override: Option<&str>,
) -> Result<()> {
    // Detect runtime
    let runtime = detect_runtime(runtime_override)?;

    // Determine agent (interactive selection if None)
    let agent = match agent {
        Some(a) => a,
        None => select_agent_interactive()?,
    };

    // Load agent config
    let (config, _is_legacy) = load_agent_config(&agent)?;

    // Ensure image is built
    ensure_image_built(runtime, &agent)?;

    // Check for existing container (new naming then legacy)
    let container_name_new = format!("klotho-session-{}-{}", agent, name);

    let existing_container = find_container(runtime, &name)?;

    if let Some(container_name) = existing_container {
        // Container exists - check if running
        let status = container_status(runtime, &container_name)?;

        match status {
            ContainerStatus::Running => {
                println!("Attaching to existing session '{}'...", name);
                return attach_zellij(runtime, &container_name, &name, &config);
            }
            ContainerStatus::Stopped => {
                println!("Starting stopped session '{}'...", name);
                start_container(runtime, &container_name)?;
                std::thread::sleep(std::time::Duration::from_secs(1));
                return attach_zellij(runtime, &container_name, &name, &config);
            }
            ContainerStatus::NotFound => {
                // Fall through to create new container
            }
        }
    }

    // Create new container
    println!("Creating new session '{}'...", name);

    // Resolve paths (default to cwd if empty)
    let resolved_paths = if paths.is_empty() {
        vec![env::current_dir().context("Failed to get current directory")?]
    } else {
        paths
            .iter()
            .map(|p| PathBuf::from(p).canonicalize())
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("Failed to resolve project path")?
    };

    // Build mount arguments
    let mut mount_args = Vec::new();

    // Project paths with :Z for SELinux
    for (i, path) in resolved_paths.iter().enumerate() {
        let mount_point = if resolved_paths.len() == 1 {
            "/workspace".to_string()
        } else {
            format!("/workspace{}", i + 1)
        };
        mount_args.push("-v".to_string());
        mount_args.push(format!("{}:{}:Z", path.display(), mount_point));
    }

    // KLOTHO_KOB with legacy AGENT_SESSION_KOB fallback
    if let Ok(kob) = env::var("KLOTHO_KOB") {
        mount_args.push("-v".to_string());
        mount_args.push(format!("{}:/home/agent/.klotho:Z", kob));
    } else if let Ok(kob) = env::var("AGENT_SESSION_KOB") {
        eprintln!(
            "note: AGENT_SESSION_KOB is deprecated, use KLOTHO_KOB instead"
        );
        mount_args.push("-v".to_string());
        mount_args.push(format!("{}:/home/agent/.klotho:Z", kob));
    }

    // KLOTHO_MOUNTS with legacy fallback
    let mounts_var = if let Ok(mounts) = env::var("KLOTHO_MOUNTS") {
        Some(mounts)
    } else if let Ok(mounts) = env::var("AGENT_SESSION_EXTRA_MOUNTS") {
        eprintln!(
            "note: AGENT_SESSION_EXTRA_MOUNTS is deprecated, use KLOTHO_MOUNTS instead"
        );
        Some(mounts)
    } else {
        None
    };

    if let Some(mounts) = mounts_var {
        for mount in mounts.split(',') {
            let mount = mount.trim();
            if !mount.is_empty() {
                mount_args.push("-v".to_string());
                mount_args.push(mount.to_string());
            }
        }
    }

    // Optional mounts (if they exist)
    let home = env::var("HOME").unwrap_or_else(|_| "/home/agent".to_string());
    let optional_mounts = vec![
        (format!("{}/.claude", home), "/home/agent/.claude:Z"),
        (
            format!("{}/.config/opencode", home),
            "/home/agent/.config/opencode:Z",
        ),
        (
            format!("{}/.config/zellij", home),
            "/home/agent/.config/zellij:Z",
        ),
    ];

    for (src, dst) in optional_mounts {
        if PathBuf::from(&src).exists() {
            mount_args.push("-v".to_string());
            mount_args.push(format!("{}:{}", src, dst));
        }
    }

    // Always mount ~/.claude.json if it exists
    let claude_json = format!("{}/.claude.json", home);
    if PathBuf::from(&claude_json).exists() {
        mount_args.push("-v".to_string());
        mount_args.push(format!("{}:/home/agent/.claude.json:Z", claude_json));
    }

    // Get image name (prefer new, fallback to legacy)
    let image_name = get_image_name(runtime, &agent)?;

    // Get working directory (first mount point)
    let workdir = if resolved_paths.len() == 1 {
        "/workspace".to_string()
    } else {
        "/workspace1".to_string()
    };

    // Run podman run with all mounts
    // Use keep-alive loop so container stays running for exec attachment
    let mut cmd = runtime.command();
    cmd.arg("run")
        .arg("-d")
        .arg("--name")
        .arg(&container_name_new)
        .arg("--userns=keep-id")
        .arg("--workdir")
        .arg(&workdir)
        .args(&mount_args)
        .arg(&image_name)
        .args(["bash", "-c", "trap 'exit 0' TERM; while :; do sleep 1; done"]);

    let output = cmd.output().context("Failed to create container")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create container: {}", stderr);
    }

    println!(
        "{} Created session '{}' → {}",
        "✓".green(),
        name.bold(),
        container_name_new.cyan()
    );

    // Give container a moment to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Attach to zellij
    attach_zellij(runtime, &container_name_new, &name, &config)
}

/// Select agent interactively
fn select_agent_interactive() -> Result<String> {
    let available_agents = if resources::should_use_embedded() {
        resources::list_embedded_agents()
    } else {
        agent::discover_agents(&PathBuf::from("."))?
    };

    if available_agents.is_empty() {
        bail!("No agents found");
    }

    if available_agents.len() == 1 {
        return Ok(available_agents[0].clone());
    }

    let selection = Select::new()
        .with_prompt("Select agent")
        .items(&available_agents)
        .default(0)
        .interact()?;

    Ok(available_agents[selection].clone())
}

/// Ensure image is built, prompt to build if missing
fn ensure_image_built(runtime: Runtime, agent: &str) -> Result<()> {
    if image_exists(runtime, agent)? {
        return Ok(());
    }

    // Image doesn't exist - prompt to build
    eprintln!(
        "{} Image for agent '{}' not found",
        "!".yellow(),
        agent.bold()
    );

    let should_build = dialoguer::Confirm::new()
        .with_prompt("Build now?")
        .default(false)
        .interact()?;

    if !should_build {
        bail!("Cannot start session without built image. Run: klotho build {}", agent);
    }

    // Build the image
    build::run_build(runtime, agent, false)?;

    Ok(())
}

/// Attach to zellij session in container
fn attach_zellij(
    runtime: Runtime,
    container_name: &str,
    session_name: &str,
    config: &AgentConfig,
) -> Result<()> {
    // Check if zellij session exists
    let check = Command::new(runtime.as_str())
        .args(["exec", container_name, "zellij", "list-sessions"])
        .output()
        .context("Failed to list zellij sessions")?;

    let stdout = String::from_utf8_lossy(&check.stdout);
    // Strip ANSI codes for comparison
    let clean_output: String = stdout
        .chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect();
    let session_exists = clean_output
        .lines()
        .any(|line| line.trim().starts_with(session_name));

    // Build the attach/create command
    let zellij_cmd = if session_exists {
        // Attach to existing session
        format!(
            "zellij attach '{}'; zellij list-sessions 2>/dev/null | sed 's/\\x1b\\[[0-9;]*m//g' | grep -q '^{} ' || exec {}",
            session_name, session_name, config.shell
        )
    } else {
        // Create new session with agent wrapper
        format!(
            "zellij -s '{}'; zellij list-sessions 2>/dev/null | sed 's/\\x1b\\[[0-9;]*m//g' | grep -q '^{} ' || exec {}",
            session_name, session_name, config.shell
        )
    };

    // Run interactive exec
    let shell_env = format!("/home/agent/.local/bin/{}-session", config.name);
    let mut cmd = Command::new(runtime.as_str());
    cmd.args(["exec", "-it"]);
    cmd.args(["-e", &format!("SHELL={}", shell_env)]);
    cmd.args(["-e", &format!("AGENT_LAUNCH_CMD={}", config.launch_cmd)]);
    cmd.args([container_name, "bash", "-c", &zellij_cmd]);

    // This is interactive - inherit stdio for TTY
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let status = cmd.status().context("Failed to attach to container")?;

    if !status.success() {
        bail!("Failed to attach to session");
    }

    Ok(())
}
