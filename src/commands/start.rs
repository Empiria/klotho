use anyhow::{bail, Context, Result};
use dialoguer::Select;
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::agent::{self, AgentConfig};
use crate::commands::{build, mobile};
use crate::config::{load_agent_config, ResolvedConfig};
use crate::container::{
    container_status, detect_runtime, ensure_network, find_container, get_image_name,
    hapi_container_name, image_exists, start_container, ContainerStatus, Runtime, KLOTHO_NETWORK,
};
use crate::project_config::resolve_credentials;
use crate::resources;

pub fn run(
    agent: Option<String>,
    name: String,
    paths: Vec<String>,
    runtime_override: Option<&str>,
) -> Result<()> {
    // Load global and project configs early
    let global_config = crate::config::load_global_config()?;
    let project_config = crate::project_config::load_project_config(&std::env::current_dir()?)?;
    let resolved = crate::config::merge_configs(&global_config, &project_config);

    // Detect runtime (use resolved config if no override)
    let effective_runtime = runtime_override.or(resolved.runtime.as_deref());
    let runtime = detect_runtime(effective_runtime)?;

    // Determine agent (use resolved default_agent as fallback)
    let agent = match agent {
        Some(a) => a,
        None => {
            if let Some(default) = &resolved.default_agent {
                default.clone()
            } else {
                select_agent_interactive()?
            }
        }
    };

    // Load agent config
    let config = load_agent_config(&agent)?;

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
                return attach_zellij(runtime, &container_name, &name, &config, &resolved);
            }
            ContainerStatus::Stopped => {
                println!("Starting stopped session '{}'...", name);
                start_container(runtime, &container_name)?;

                std::thread::sleep(std::time::Duration::from_secs(1));
                return attach_zellij(runtime, &container_name, &name, &config, &resolved);
            }
            ContainerStatus::NotFound => {
                // Fall through to create new container
            }
        }
    }

    // Create new container
    println!("Creating new session '{}'...", name);

    // Ensure klotho network exists so session containers can communicate with hapi
    if let Err(e) = ensure_network(runtime, KLOTHO_NETWORK) {
        eprintln!("warning: failed to ensure klotho network: {}", e);
    }

    // Derive named_workdir early (before volume and MCP mounting)
    let named_workdir = resolved
        .project
        .as_ref()
        .and_then(|p| p.workdir.as_deref())
        .map(|w| w.to_string())
        .unwrap_or_else(|| format!("/home/agent/{}", &name));

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

    // Mount primary project at named_workdir so getcwd() returns the session name
    // (symlinks won't work — the kernel resolves them in getcwd)
    for (i, path) in resolved_paths.iter().enumerate() {
        let mount_point = if i == 0 {
            named_workdir.clone()
        } else {
            format!("/workspace{}", i + 1)
        };
        mount_args.push("-v".to_string());
        mount_args.push(format!("{}:{}:Z", path.display(), mount_point));
    }

    // Mount volumes from config (global + project, merged)
    for vol in &resolved.volumes {
        let (src, dest, readonly) = vol.resolve();
        let src_path = PathBuf::from(&src);
        if !src_path.exists() {
            eprintln!("warning: volume source does not exist, skipping: {}", src);
            continue;
        }
        let canonical = src_path
            .canonicalize()
            .context(format!("failed to resolve volume path: {}", src))?;
        let suffix = if readonly { ":ro" } else { ":Z" };
        mount_args.push("-v".to_string());
        mount_args.push(format!("{}:{}{}", canonical.display(), dest, suffix));
    }

    // Agent-specific optional mounts (only if mount_host_config is enabled)
    if resolved.mount_host_config {
        for vol in &config.optional_mounts {
            let (src, dest, _readonly) = vol.resolve();
            if PathBuf::from(&src).exists() {
                mount_args.push("-v".to_string());
                mount_args.push(format!("{}:{}:Z", src, dest));
            }
        }
    }

    // Zellij config mount (built-in, not agent-specific)
    let home = env::var("HOME").unwrap_or_else(|_| "/home/agent".to_string());
    let zellij_config = format!("{}/.config/zellij", home);
    if PathBuf::from(&zellij_config).exists() {
        mount_args.push("-v".to_string());
        mount_args.push(format!("{}:/home/agent/.config/zellij:Z", zellij_config));
    }

    // Always mount ~/.claude.json if it exists (infrastructure, not agent-specific)
    let claude_json = format!("{}/.claude.json", home);
    if PathBuf::from(&claude_json).exists() {
        mount_args.push("-v".to_string());
        mount_args.push(format!("{}:/home/agent/.claude.json:Z", claude_json));
    }

    // Generate and mount MCP config for the agent at start time
    let mcp_servers = crate::project_config::resolve_mcp_servers(&resolved.mcp, &agent);

    if !mcp_servers.is_empty() {
        let temp_dir = std::env::temp_dir().join("klotho-mcp");
        std::fs::create_dir_all(&temp_dir)?;

        match agent.as_str() {
            "opencode" => {
                let json = crate::project_config::mcp_to_opencode_json(&mcp_servers);
                let json_path = temp_dir.join("opencode.json");
                std::fs::write(&json_path, serde_json::to_string_pretty(&json)?)?;
                mount_args.push("-v".to_string());
                mount_args.push(format!(
                    "{}:/home/agent/.config/opencode/opencode.json:Z",
                    json_path.display()
                ));
            }
            "claude" => {
                let json = crate::project_config::mcp_to_claude_json(&mcp_servers);
                let json_path = temp_dir.join(".mcp.json");
                std::fs::write(&json_path, serde_json::to_string_pretty(&json)?)?;
                mount_args.push("-v".to_string());
                mount_args.push(format!(
                    "{}:{}/.mcp.json:Z",
                    json_path.display(),
                    named_workdir
                ));
            }
            _ => {
                eprintln!(
                    "warning: MCP config translation not supported for agent '{}'",
                    agent
                );
            }
        }
    }

    // Get credential env args (API keys from config)
    let credential_env_args = get_credential_env_args(&agent, &resolved)?;

    // Resolve skills for this agent and prepare env args
    let skills_env_args = get_skills_env_args(&agent, &resolved)?;

    // Get image name (prefer new, fallback to legacy)
    let image_name = get_image_name(runtime, &agent)?;

    // Backward-compat symlink name (/workspace or /workspace1)
    let compat_workdir = if resolved_paths.len() == 1 {
        "/workspace"
    } else {
        "/workspace1"
    };

    // Prepare hapi env vars if hub is running
    let hapi_name = hapi_container_name();
    let hapi_env_args = if let Ok(ContainerStatus::Running) = container_status(runtime, &hapi_name)
    {
        if let Some(token) = mobile::get_cli_token(runtime, &hapi_name) {
            vec![
                "-e".to_string(),
                format!("CLI_API_TOKEN={}", token),
                "-e".to_string(),
                "HAPI_API_URL=http://klotho-hapi:3006".to_string(),
            ]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Run podman run with all mounts
    // Use keep-alive loop so container stays running for exec attachment
    let mut cmd = runtime.command();
    cmd.arg("run")
        .arg("-d")
        .arg("--name")
        .arg(&container_name_new)
        .arg("--label=klotho=true")
        .arg("--userns=keep-id")
        .args(["--network", KLOTHO_NETWORK])
        .arg("--workdir")
        .arg(&named_workdir)
        .args(&mount_args)
        .args(&hapi_env_args)
        .args(&credential_env_args)
        .args(&skills_env_args)
        .arg(&image_name);

    // Create /workspace symlink pointing to the named workdir for backward compat
    let startup_cmd = format!(
        "ln -sfn {} {} 2>/dev/null; trap 'exit 0' TERM; while :; do sleep 1; done",
        named_workdir, compat_workdir
    );
    cmd.args(["bash", "-c", &startup_cmd]);

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
    attach_zellij(runtime, &container_name_new, &name, &config, &resolved)
}

/// Get environment variable arguments for agent credentials
fn get_credential_env_args(agent: &str, resolved: &ResolvedConfig) -> Result<Vec<String>> {
    let mut env_args = Vec::new();

    if let Some(creds) = resolved.agents.get(agent) {
        // Resolve env var references in the API key
        let resolved_creds = resolve_credentials(creds)?;

        if let Some(ref api_key) = resolved_creds.api_key {
            // Map agent name to expected env var
            let env_var = match agent {
                "claude" => "ANTHROPIC_API_KEY",
                "opencode" => {
                    // OpenCode can use multiple providers, default to Anthropic
                    // User can override with explicit OPENAI_API_KEY in config
                    "ANTHROPIC_API_KEY"
                }
                _ => {
                    // For unknown agents, use generic pattern
                    eprintln!(
                        "warning: unknown agent '{}', skipping credential injection",
                        agent
                    );
                    return Ok(env_args);
                }
            };
            env_args.push("-e".to_string());
            env_args.push(format!("{}={}", env_var, api_key));
        }
    }

    Ok(env_args)
}

/// Get environment variable arguments for skills
fn get_skills_env_args(agent: &str, resolved: &ResolvedConfig) -> Result<Vec<String>> {
    use std::collections::HashMap;

    let mut env_args = Vec::new();

    // Resolve skills for this agent
    let empty_skills = HashMap::new();
    let agent_skills = resolved
        .agents
        .get(agent)
        .map(|a| &a.skills)
        .unwrap_or(&empty_skills);

    let skills =
        crate::project_config::resolve_skills_for_agent(&resolved.skills, agent_skills, agent);

    // Always pass agent name for entrypoint context
    env_args.push("-e".to_string());
    env_args.push(format!("KLOTHO_AGENT={}", agent));

    // Encode skills as JSON for entrypoint
    if !skills.is_empty() {
        let skills_json =
            serde_json::to_string(&skills).context("Failed to serialize skills to JSON")?;
        env_args.push("-e".to_string());
        env_args.push(format!("KLOTHO_SKILLS={}", skills_json));
    }

    Ok(env_args)
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
        bail!(
            "Cannot start session without built image. Run: klotho build {}",
            agent
        );
    }

    // Build the image (no custom packages from start command)
    build::run_build(runtime, agent, &[], false)?;

    Ok(())
}

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    // ANSI escape sequences follow pattern: ESC [ <params> m
    // where ESC is \x1b, params are digits/semicolons
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Check if this is start of ANSI sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                              // Skip until we hit 'm' (or end of string)
                while let Some(ch) = chars.next() {
                    if ch == 'm' {
                        break;
                    }
                }
                continue;
            }
        }
        result.push(ch);
    }

    result
}

/// Attach to zellij session in container
fn attach_zellij(
    runtime: Runtime,
    container_name: &str,
    session_name: &str,
    config: &AgentConfig,
    resolved: &ResolvedConfig,
) -> Result<()> {
    // Check if zellij session exists
    let check = Command::new(runtime.as_str())
        .args(["exec", container_name, "zellij", "list-sessions"])
        .output()
        .context("Failed to list zellij sessions")?;

    let stdout = String::from_utf8_lossy(&check.stdout);
    // Strip ANSI codes for comparison (regex pattern: \x1b\[[0-9;]*m)
    let clean_output = strip_ansi_codes(&stdout);
    let session_exists = clean_output
        .lines()
        .any(|line| line.trim().starts_with(session_name));

    // Ensure /home/agent/<name> exists and cd into it.
    // New containers mount directly there; old containers need a symlink for best-effort compat.
    let symlink_setup = format!(
        "test -e /home/agent/{name} || ln -sfn \"$(readlink -f .)\" /home/agent/{name}; \
         cd /home/agent/{name} 2>/dev/null; ",
        name = session_name
    );

    // Build the attach/create command
    let zellij_cmd = if session_exists {
        // Attach to existing session
        format!(
            "{setup}zellij attach '{}'; zellij list-sessions 2>/dev/null | sed 's/\\x1b\\[[0-9;]*m//g' | grep -q '^{} ' || exec {}",
            session_name, session_name, config.shell, setup = symlink_setup
        )
    } else {
        // Create new session with agent wrapper
        format!(
            "{setup}zellij -s '{}'; zellij list-sessions 2>/dev/null | sed 's/\\x1b\\[[0-9;]*m//g' | grep -q '^{} ' || exec {}",
            session_name, session_name, config.shell, setup = symlink_setup
        )
    };

    // Inject hapi env vars if hub is running (benefits sessions created before hub)
    let hapi_name = hapi_container_name();
    let mut hapi_env_args = Vec::new();
    if let Ok(ContainerStatus::Running) = container_status(runtime, &hapi_name) {
        if let Some(token) = mobile::get_cli_token(runtime, &hapi_name) {
            hapi_env_args.extend_from_slice(&[
                "-e".to_string(),
                format!("CLI_API_TOKEN={}", token),
                "-e".to_string(),
                "HAPI_API_URL=http://klotho-hapi:3006".to_string(),
            ]);
        }
    }

    // Inject credential env args for existing sessions
    let credential_env_args = get_credential_env_args(&config.name, resolved)?;

    // Run interactive exec
    let shell_env = format!("/home/agent/.local/bin/{}-session", config.name);
    let mut cmd = Command::new(runtime.as_str());
    cmd.args(["exec", "-it"]);
    for arg in &hapi_env_args {
        cmd.arg(arg);
    }
    for arg in &credential_env_args {
        cmd.arg(arg);
    }
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
