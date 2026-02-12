use anyhow::{bail, Context, Result};
use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Agent credentials configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct AgentCredentials {
    /// API key for this agent, supports ${ENV_VAR} syntax
    #[serde(default)]
    pub api_key: Option<String>,
}

/// Volume mount specification - shared by agent config, project config, and global config
/// Supports both simple string format and detailed format with options
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum VolumeSpec {
    /// Detailed mount specification with source, destination, and optional readonly flag
    Detailed {
        src: String,
        dest: String,
        #[serde(default)]
        readonly: bool,
    },
    /// Simple mount specification as a single string (e.g., "/host/path:/container/path")
    Simple(String),
}

impl VolumeSpec {
    /// Resolve volume to (src, dest, readonly) tuple with tilde expansion
    pub fn resolve(&self) -> (String, String, bool) {
        match self {
            VolumeSpec::Simple(path) => {
                let expanded = expand_tilde(path);
                (expanded.clone(), expanded, false)
            }
            VolumeSpec::Detailed {
                src,
                dest,
                readonly,
            } => (expand_tilde(src), expand_tilde(dest), *readonly),
        }
    }
}

/// Expand ~ to $HOME in path
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

/// Resolve ${VAR_NAME} references in a string
/// Uses regex-lite which is already in Cargo.toml
pub fn resolve_env_vars(value: &str) -> Result<String> {
    let re = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap();
    let mut result = value.to_string();

    for cap in re.captures_iter(value) {
        let full_match = &cap[0]; // ${VAR_NAME}
        let var_name = &cap[1]; // VAR_NAME

        let var_value = std::env::var(var_name).context(format!(
            "Environment variable {} is not set (referenced as {})",
            var_name, full_match
        ))?;

        result = result.replace(full_match, &var_value);
    }

    Ok(result)
}

/// Resolve credentials, failing on unresolved env vars
pub fn resolve_credentials(creds: &AgentCredentials) -> Result<AgentCredentials> {
    Ok(AgentCredentials {
        api_key: creds
            .api_key
            .as_ref()
            .map(|k| resolve_env_vars(k))
            .transpose()?,
    })
}

/// Resolve which MCP servers apply to an agent
/// If agent-specific section exists (e.g., mcp.claude), use ONLY that section (replaces shared entirely).
/// Otherwise, use shared servers.
pub fn resolve_mcp_servers(mcp: &McpConfig, agent: &str) -> HashMap<String, McpServerConfig> {
    match agent {
        "claude" => {
            if !mcp.claude.is_empty() {
                mcp.claude.clone()
            } else {
                mcp.servers.clone()
            }
        }
        "opencode" => {
            if !mcp.opencode.is_empty() {
                mcp.opencode.clone()
            } else {
                mcp.servers.clone()
            }
        }
        _ => mcp.servers.clone(),
    }
}

/// Convert MCP servers to Claude Desktop JSON format (.mcp.json)
/// Format: { "mcpServers": { "server-name": { "command": "cmd", "args": ["arg1"], "env": {...} } } }
pub fn mcp_to_claude_json(servers: &HashMap<String, McpServerConfig>) -> serde_json::Value {
    let mut mcp_servers = serde_json::Map::new();

    for (name, config) in servers {
        let mut server_obj = serde_json::Map::new();
        server_obj.insert(
            "command".to_string(),
            serde_json::Value::String(config.command.clone()),
        );

        if !config.args.is_empty() {
            server_obj.insert(
                "args".to_string(),
                serde_json::Value::Array(
                    config
                        .args
                        .iter()
                        .map(|a| serde_json::Value::String(a.clone()))
                        .collect(),
                ),
            );
        }

        if !config.env.is_empty() {
            let env_obj: serde_json::Map<String, serde_json::Value> = config
                .env
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            server_obj.insert("env".to_string(), serde_json::Value::Object(env_obj));
        }

        mcp_servers.insert(name.clone(), serde_json::Value::Object(server_obj));
    }

    let mut result = serde_json::Map::new();
    result.insert(
        "mcpServers".to_string(),
        serde_json::Value::Object(mcp_servers),
    );
    serde_json::Value::Object(result)
}

/// Convert MCP servers to OpenCode JSON format (opencode.json)
/// Format: { "mcp": { "server-name": { "type": "local", "command": ["cmd", "arg1"], "enabled": true } } }
pub fn mcp_to_opencode_json(servers: &HashMap<String, McpServerConfig>) -> serde_json::Value {
    let mut mcp_obj = serde_json::Map::new();

    for (name, config) in servers {
        let mut server_obj = serde_json::Map::new();
        server_obj.insert(
            "type".to_string(),
            serde_json::Value::String("local".to_string()),
        );

        // Build command array: [command, arg1, arg2, ...]
        let mut command_array = vec![serde_json::Value::String(config.command.clone())];
        command_array.extend(
            config
                .args
                .iter()
                .map(|a| serde_json::Value::String(a.clone())),
        );
        server_obj.insert(
            "command".to_string(),
            serde_json::Value::Array(command_array),
        );

        server_obj.insert("enabled".to_string(), serde_json::Value::Bool(true));

        mcp_obj.insert(name.clone(), serde_json::Value::Object(server_obj));
    }

    let mut result = serde_json::Map::new();
    result.insert("mcp".to_string(), serde_json::Value::Object(mcp_obj));
    serde_json::Value::Object(result)
}

/// MCP server configuration
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// MCP configuration with shared and agent-specific server sections
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct McpConfig {
    /// Shared servers (available to all agents)
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
    /// Claude-specific servers
    #[serde(default)]
    pub claude: HashMap<String, McpServerConfig>,
    /// OpenCode-specific servers
    #[serde(default)]
    pub opencode: HashMap<String, McpServerConfig>,
}

/// Project metadata from [project] section
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ProjectMeta {
    pub agent: Option<String>,
    pub name: Option<String>,
    pub workdir: Option<String>,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct KlothoConfig {
    #[serde(default)]
    pub packages: Option<Packages>,
    #[serde(default)]
    pub project: Option<ProjectMeta>,
    #[serde(default)]
    pub volumes: Vec<VolumeSpec>,
    #[serde(default)]
    pub mcp: Option<McpConfig>,
    #[serde(default)]
    pub agents: HashMap<String, AgentCredentials>,
    /// Opt-in to mount host config directories (~/.claude, ~/.config/opencode)
    #[serde(default)]
    pub mount_host_config: bool,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Packages {
    #[serde(default)]
    pub apt: HashMap<String, String>,
    #[serde(default)]
    pub pip: HashMap<String, String>,
    #[serde(default)]
    pub npm: HashMap<String, String>,
    #[serde(default)]
    pub cargo: HashMap<String, String>,
}

impl Packages {
    pub fn has_packages(&self) -> bool {
        !self.apt.is_empty()
            || !self.pip.is_empty()
            || !self.npm.is_empty()
            || !self.cargo.is_empty()
    }
}

/// Load .klotho.toml from project directory. Returns empty config if file doesn't exist.
pub fn load_project_config(project_path: &Path) -> Result<KlothoConfig> {
    let config_path = project_path.join(".klotho.toml");

    if !config_path.exists() {
        // No config file is OK - return empty config
        return Ok(KlothoConfig::default());
    }

    let contents = std::fs::read_to_string(&config_path)
        .context(format!("Failed to read {}", config_path.display()))?;

    toml::from_str(&contents).context(format!("Failed to parse {} as TOML", config_path.display()))
}

/// Validate package name against safe charset [a-zA-Z0-9._-+]
pub fn validate_package_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Package name cannot be empty");
    }

    let valid_chars = Regex::new(r"^[a-zA-Z0-9._\-+@/]+$").unwrap();
    if !valid_chars.is_match(name) {
        bail!(
            "Invalid package name '{}': must contain only alphanumeric characters, dots, hyphens, underscores, plus signs, @, or /",
            name
        );
    }

    Ok(())
}

/// Validate all package names across all package managers
pub fn validate_all_packages(packages: &Packages) -> Result<()> {
    let mut errors = Vec::new();

    for name in packages.apt.keys() {
        if let Err(e) = validate_package_name(name) {
            errors.push(e.to_string());
        }
    }

    for name in packages.pip.keys() {
        if let Err(e) = validate_package_name(name) {
            errors.push(e.to_string());
        }
    }

    for name in packages.npm.keys() {
        if let Err(e) = validate_package_name(name) {
            errors.push(e.to_string());
        }
    }

    for name in packages.cargo.keys() {
        if let Err(e) = validate_package_name(name) {
            errors.push(e.to_string());
        }
    }

    if !errors.is_empty() {
        bail!("Package validation failed:\n  {}", errors.join("\n  "));
    }

    Ok(())
}

/// Merge CLI --install flags into packages. Format: "manager:package" or "manager:package=version"
/// Note: version can include operators like ==, >=, ^, etc. - they're preserved and passed to package manager
pub fn merge_cli_installs(packages: &mut Packages, cli_installs: &[String]) -> Result<()> {
    for flag in cli_installs {
        let (manager, rest) = flag
            .split_once(':')
            .context(format!("Invalid --install flag '{}': must be format 'manager:package' or 'manager:package=version'", flag))?;

        // Parse package and version. Examples:
        // "gcc" -> ("gcc", "*")
        // "gcc=11" -> ("gcc", "11")
        // "pytest==7.0" -> ("pytest", "==7.0")  (both = are version operator)
        // "typescript=^5.0" -> ("typescript", "^5.0")  (first = is delimiter)
        //
        // Strategy: Split on first =. If the part after the = starts with another
        // operator (=, <, >, etc.), include that first = in the version string.
        let (package, version) = if let Some((pkg, after_eq)) = rest.split_once('=') {
            // Check if what follows the = is itself an operator (making ==, >=, etc.)
            if after_eq.starts_with('=') || after_eq.starts_with('<') || after_eq.starts_with('>') {
                // This is a double operator like ==, =<, =>
                // Include the first = in the version
                (pkg, format!("={}", after_eq))
            } else {
                // Single = is just a delimiter
                (pkg, after_eq.to_string())
            }
        } else {
            (rest, "*".to_string())
        };

        let map = match manager {
            "apt" => &mut packages.apt,
            "pip" => &mut packages.pip,
            "npm" => &mut packages.npm,
            "cargo" => &mut packages.cargo,
            _ => bail!(
                "Unknown package manager '{}' in flag '{}'. Supported managers: apt, pip, npm, cargo",
                manager,
                flag
            ),
        };

        map.entry(package.to_string()).or_insert(version);
    }

    Ok(())
}

/// Generate Containerfile RUN commands for installing packages
/// Returns a vector of RUN command strings that can be inserted into a Containerfile
pub fn generate_install_commands(packages: &Packages) -> Vec<String> {
    let mut commands = Vec::new();
    let mut packages = packages.clone();

    // Check for known runtime recipes first and generate their installers
    // Remove these keys from the maps so they're not also passed to standard installers

    // Rust/rustup recipe
    if packages.cargo.contains_key("rust") || packages.cargo.contains_key("rustup") {
        commands.push(
            "RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && . \"$HOME/.cargo/env\"".to_string()
        );
        packages.cargo.remove("rust");
        packages.cargo.remove("rustup");
    }

    // Node/nvm recipe
    if packages.npm.contains_key("node") || packages.npm.contains_key("nvm") {
        commands.push(
            "RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash && . \"$HOME/.nvm/nvm.sh\" && nvm install --lts".to_string()
        );
        packages.npm.remove("node");
        packages.npm.remove("nvm");
    }

    // APT packages - install system deps first
    if !packages.apt.is_empty() {
        let mut pkg_list: Vec<String> = packages
            .apt
            .iter()
            .map(|(name, version)| {
                if version == "*" {
                    name.clone()
                } else {
                    format!("{}={}*", name, version)
                }
            })
            .collect();
        pkg_list.sort(); // Alphabetical for deterministic output

        commands.push(format!(
            "RUN apt-get update && apt-get install -y --no-install-recommends {} && rm -rf /var/lib/apt/lists/*",
            pkg_list.join(" \\\n    ")
        ));
    }

    // PIP packages
    if !packages.pip.is_empty() {
        let pkg_specs: Vec<String> = packages
            .pip
            .iter()
            .map(|(name, version)| {
                if version == "*" {
                    name.clone()
                } else {
                    format!("{}{}", name, version)
                }
            })
            .collect();

        commands.push(format!(
            "RUN pip3 install --no-cache-dir {}",
            pkg_specs.join(" ")
        ));
    }

    // NPM packages (global)
    if !packages.npm.is_empty() {
        let pkg_specs: Vec<String> = packages
            .npm
            .iter()
            .map(|(name, version)| {
                if version == "*" {
                    name.clone()
                } else {
                    format!("{}@{}", name, version)
                }
            })
            .collect();

        commands.push(format!("RUN npm install -g {}", pkg_specs.join(" ")));
    }

    // Cargo packages - each gets its own RUN for better caching
    for (name, version) in &packages.cargo {
        if version == "*" {
            commands.push(format!("RUN cargo install {}", name));
        } else {
            commands.push(format!("RUN cargo install {} --version {}", name, version));
        }
    }

    commands
}

/// Generate a complete Containerfile snippet for custom packages
/// Returns empty string if no packages
pub fn generate_containerfile_snippet(packages: &Packages) -> String {
    let commands = generate_install_commands(packages);
    if commands.is_empty() {
        String::new()
    } else {
        commands.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_toml_all_sections() {
        let toml_content = r#"
[packages.apt]
gcc = "*"
python3 = "3.11"

[packages.pip]
pytest = ">=7.0"
requests = "*"

[packages.npm]
typescript = "^5.0"

[packages.cargo]
ripgrep = "*"
"#;

        let config: KlothoConfig = toml::from_str(toml_content).unwrap();
        let packages = config.packages.unwrap();

        assert_eq!(packages.apt.get("gcc"), Some(&"*".to_string()));
        assert_eq!(packages.apt.get("python3"), Some(&"3.11".to_string()));
        assert_eq!(packages.pip.get("pytest"), Some(&">=7.0".to_string()));
        assert_eq!(packages.npm.get("typescript"), Some(&"^5.0".to_string()));
        assert_eq!(packages.cargo.get("ripgrep"), Some(&"*".to_string()));
    }

    #[test]
    fn test_parse_partial_sections() {
        let toml_content = r#"
[packages.apt]
gcc = "*"
"#;

        let config: KlothoConfig = toml::from_str(toml_content).unwrap();
        let packages = config.packages.unwrap();

        assert_eq!(packages.apt.len(), 1);
        assert_eq!(packages.pip.len(), 0);
        assert_eq!(packages.npm.len(), 0);
        assert_eq!(packages.cargo.len(), 0);
    }

    #[test]
    fn test_parse_empty_toml() {
        let toml_content = "";
        let config: KlothoConfig = toml::from_str(toml_content).unwrap();
        assert!(config.packages.is_none() || config.packages.unwrap().has_packages() == false);
    }

    #[test]
    fn test_parse_no_packages_section() {
        let toml_content = r#"
[some_other_section]
key = "value"
"#;
        let config: KlothoConfig = toml::from_str(toml_content).unwrap();
        assert!(config.packages.is_none() || config.packages.unwrap().has_packages() == false);
    }

    #[test]
    fn test_missing_file_returns_default() {
        let result = load_project_config(Path::new("/nonexistent/path"));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.packages.is_none() || config.packages.unwrap().has_packages() == false);
    }

    #[test]
    fn test_invalid_toml_returns_error() {
        use std::io::Write;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_invalid.klotho.toml");

        std::fs::File::create(&test_file)
            .unwrap()
            .write_all(b"[packages.apt\ninvalid toml")
            .unwrap();

        let parent_dir = test_file.parent().unwrap();

        // Create .klotho.toml in the parent directory
        let config_file = parent_dir.join(".klotho.toml");
        std::fs::write(&config_file, b"[packages.apt\ninvalid toml").unwrap();

        let result = load_project_config(parent_dir);
        assert!(result.is_err());

        // Cleanup
        std::fs::remove_file(config_file).ok();
    }

    #[test]
    fn test_validate_valid_package_names() {
        assert!(validate_package_name("gcc").is_ok());
        assert!(validate_package_name("python3").is_ok());
        assert!(validate_package_name("build-essential").is_ok());
        assert!(validate_package_name("node.js").is_ok());
        assert!(validate_package_name("libssl-dev").is_ok());
        assert!(validate_package_name("g++").is_ok());
        assert!(validate_package_name("@types/node").is_ok());
        assert!(validate_package_name("@babel/core").is_ok());
    }

    #[test]
    fn test_validate_invalid_package_names() {
        assert!(validate_package_name("pack;age").is_err());
        assert!(validate_package_name("pack|age").is_err());
        assert!(validate_package_name("pack&&age").is_err());
        assert!(validate_package_name("$(cmd)").is_err());
        assert!(validate_package_name("`cmd`").is_err());
        assert!(validate_package_name("pack age").is_err());
        assert!(validate_package_name("").is_err());
    }

    #[test]
    fn test_cli_flag_parsing() {
        let mut packages = Packages::default();

        merge_cli_installs(
            &mut packages,
            &[
                "apt:gcc".to_string(),
                "pip:pytest==7.0".to_string(),
                "npm:typescript=^5.0".to_string(),
                "cargo:ripgrep".to_string(),
            ],
        )
        .unwrap();

        assert_eq!(packages.apt.get("gcc"), Some(&"*".to_string()));
        assert_eq!(packages.pip.get("pytest"), Some(&"==7.0".to_string()));
        assert_eq!(packages.npm.get("typescript"), Some(&"^5.0".to_string()));
        assert_eq!(packages.cargo.get("ripgrep"), Some(&"*".to_string()));
    }

    #[test]
    fn test_cli_merge_is_additive() {
        let mut packages = Packages::default();
        packages.apt.insert("gcc".to_string(), "11".to_string());

        merge_cli_installs(&mut packages, &["apt:python3".to_string()]).unwrap();

        assert_eq!(packages.apt.get("gcc"), Some(&"11".to_string()));
        assert_eq!(packages.apt.get("python3"), Some(&"*".to_string()));
    }

    #[test]
    fn test_cli_merge_doesnt_replace_existing() {
        let mut packages = Packages::default();
        packages.apt.insert("gcc".to_string(), "11".to_string());

        merge_cli_installs(&mut packages, &["apt:gcc=12".to_string()]).unwrap();

        // Should keep original version since entry already exists
        assert_eq!(packages.apt.get("gcc"), Some(&"11".to_string()));
    }

    #[test]
    fn test_unknown_manager_returns_error() {
        let mut packages = Packages::default();
        let result = merge_cli_installs(&mut packages, &["unknown:package".to_string()]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown package manager"));
    }

    #[test]
    fn test_has_packages() {
        let mut packages = Packages::default();
        assert!(!packages.has_packages());

        packages.apt.insert("gcc".to_string(), "*".to_string());
        assert!(packages.has_packages());
    }

    #[test]
    fn test_validate_all_packages() {
        let mut packages = Packages::default();
        packages.apt.insert("gcc".to_string(), "*".to_string());
        packages.pip.insert("pytest".to_string(), "*".to_string());

        assert!(validate_all_packages(&packages).is_ok());

        packages
            .npm
            .insert("bad;package".to_string(), "*".to_string());
        assert!(validate_all_packages(&packages).is_err());
    }

    #[test]
    fn test_generate_apt_commands() {
        let mut packages = Packages::default();
        packages.apt.insert("gcc".to_string(), "*".to_string());
        packages
            .apt
            .insert("python3".to_string(), "3.11".to_string());

        let commands = generate_install_commands(&packages);

        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("apt-get update"));
        assert!(commands[0].contains("apt-get install"));
        assert!(commands[0].contains("gcc"));
        assert!(commands[0].contains("python3=3.11*"));
        assert!(commands[0].contains("rm -rf /var/lib/apt/lists/*"));
    }

    #[test]
    fn test_generate_pip_commands() {
        let mut packages = Packages::default();
        packages
            .pip
            .insert("pytest".to_string(), ">=7.0".to_string());
        packages.pip.insert("requests".to_string(), "*".to_string());

        let commands = generate_install_commands(&packages);

        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("pip3 install"));
        assert!(commands[0].contains("--no-cache-dir"));
        assert!(commands[0].contains("pytest>=7.0"));
        assert!(commands[0].contains("requests"));
    }

    #[test]
    fn test_generate_npm_commands() {
        let mut packages = Packages::default();
        packages
            .npm
            .insert("typescript".to_string(), "^5.0".to_string());
        packages
            .npm
            .insert("@types/node".to_string(), "*".to_string());

        let commands = generate_install_commands(&packages);

        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("npm install -g"));
        assert!(commands[0].contains("typescript@^5.0"));
        assert!(commands[0].contains("@types/node"));
    }

    #[test]
    fn test_generate_cargo_commands() {
        let mut packages = Packages::default();
        packages
            .cargo
            .insert("ripgrep".to_string(), "*".to_string());
        packages
            .cargo
            .insert("fd-find".to_string(), "8.7.0".to_string());

        let commands = generate_install_commands(&packages);

        assert_eq!(commands.len(), 2);

        // HashMap iteration order is non-deterministic, so check both commands exist
        let all_commands = commands.join(" ");
        assert!(all_commands.contains("cargo install ripgrep"));
        assert!(all_commands.contains("cargo install fd-find --version 8.7.0"));

        // Verify one has --version and one doesn't
        let has_version_count = commands.iter().filter(|c| c.contains("--version")).count();
        assert_eq!(has_version_count, 1);
    }

    #[test]
    fn test_generate_empty_packages() {
        let packages = Packages::default();
        let commands = generate_install_commands(&packages);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_rustup_recipe() {
        let mut packages = Packages::default();
        packages.cargo.insert("rustup".to_string(), "*".to_string());
        packages
            .cargo
            .insert("ripgrep".to_string(), "*".to_string());

        let commands = generate_install_commands(&packages);

        // Should have rustup installer + ripgrep
        assert_eq!(commands.len(), 2);
        assert!(commands[0].contains("https://sh.rustup.rs"));
        assert!(commands[0].contains("sh -s -- -y"));
        assert!(commands[1].contains("cargo install ripgrep"));
    }

    #[test]
    fn test_nvm_recipe() {
        let mut packages = Packages::default();
        packages.npm.insert("nvm".to_string(), "*".to_string());
        packages
            .npm
            .insert("typescript".to_string(), "*".to_string());

        let commands = generate_install_commands(&packages);

        // Should have nvm installer + typescript
        assert_eq!(commands.len(), 2);
        assert!(commands[0].contains("nvm-sh/nvm"));
        assert!(commands[0].contains("nvm install --lts"));
        assert!(commands[1].contains("npm install -g typescript"));
    }

    #[test]
    fn test_recipe_removes_from_package_map() {
        let mut packages = Packages::default();
        packages.cargo.insert("rust".to_string(), "*".to_string());

        let commands = generate_install_commands(&packages);

        // Should only have rustup installer, not a "cargo install rust"
        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("rustup.rs"));
        assert!(!commands[0].contains("cargo install"));
    }

    #[test]
    fn test_mixed_package_managers() {
        let mut packages = Packages::default();
        packages.apt.insert("gcc".to_string(), "*".to_string());
        packages.pip.insert("pytest".to_string(), "*".to_string());
        packages
            .npm
            .insert("typescript".to_string(), "*".to_string());
        packages
            .cargo
            .insert("ripgrep".to_string(), "*".to_string());

        let commands = generate_install_commands(&packages);

        // Should have all four: apt, pip, npm, cargo
        assert_eq!(commands.len(), 4);
        assert!(commands[0].contains("apt-get"));
        assert!(commands[1].contains("pip3"));
        assert!(commands[2].contains("npm"));
        assert!(commands[3].contains("cargo"));
    }

    #[test]
    fn test_containerfile_snippet_generation() {
        let mut packages = Packages::default();
        packages.apt.insert("gcc".to_string(), "*".to_string());
        packages.pip.insert("pytest".to_string(), "*".to_string());

        let snippet = generate_containerfile_snippet(&packages);

        assert!(!snippet.is_empty());
        assert!(snippet.contains("RUN apt-get"));
        assert!(snippet.contains("RUN pip3"));
        assert!(snippet.contains('\n')); // Multiple commands joined by newline
    }

    #[test]
    fn test_containerfile_snippet_empty() {
        let packages = Packages::default();
        let snippet = generate_containerfile_snippet(&packages);
        assert_eq!(snippet, "");
    }

    #[test]
    fn test_parse_project_section() {
        let toml_content = r#"
[project]
agent = "claude"
name = "myproject"
workdir = "/workspace"
"#;
        let config: KlothoConfig = toml::from_str(toml_content).unwrap();
        let project = config.project.unwrap();
        assert_eq!(project.agent, Some("claude".to_string()));
        assert_eq!(project.name, Some("myproject".to_string()));
        assert_eq!(project.workdir, Some("/workspace".to_string()));
    }

    #[test]
    fn test_parse_volumes_simple() {
        let toml_content = r#"
volumes = [
    "/home/user/libs",
    "/var/data"
]
"#;
        let config: KlothoConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.volumes.len(), 2);
        assert_eq!(
            config.volumes[0],
            VolumeSpec::Simple("/home/user/libs".to_string())
        );
        assert_eq!(
            config.volumes[1],
            VolumeSpec::Simple("/var/data".to_string())
        );
    }

    #[test]
    fn test_parse_volumes_detailed() {
        let toml_content = r#"
[[volumes]]
src = "/host/path"
dest = "/container/path"
readonly = true

[[volumes]]
src = "/host/other"
dest = "/container/other"
"#;
        let config: KlothoConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.volumes.len(), 2);

        match &config.volumes[0] {
            VolumeSpec::Detailed {
                src,
                dest,
                readonly,
            } => {
                assert_eq!(src, "/host/path");
                assert_eq!(dest, "/container/path");
                assert!(readonly);
            }
            _ => panic!("Expected Detailed variant"),
        }

        match &config.volumes[1] {
            VolumeSpec::Detailed {
                src,
                dest,
                readonly,
            } => {
                assert_eq!(src, "/host/other");
                assert_eq!(dest, "/container/other");
                assert!(!readonly);
            }
            _ => panic!("Expected Detailed variant"),
        }
    }

    #[test]
    fn test_parse_volumes_mixed() {
        let toml_content = r#"
volumes = [
    "/simple/path",
    { src = "/host/path", dest = "/container/path", readonly = true }
]
"#;
        let config: KlothoConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.volumes.len(), 2);
        assert_eq!(
            config.volumes[0],
            VolumeSpec::Simple("/simple/path".to_string())
        );

        match &config.volumes[1] {
            VolumeSpec::Detailed {
                src,
                dest,
                readonly,
            } => {
                assert_eq!(src, "/host/path");
                assert_eq!(dest, "/container/path");
                assert!(readonly);
            }
            _ => panic!("Expected Detailed variant"),
        }
    }

    #[test]
    fn test_parse_mcp_config() {
        let toml_content = r#"
[mcp.servers.context7]
command = "uvx"
args = ["@upstash/context7-mcp"]

[mcp.claude.custom-tool]
command = "/usr/bin/custom"
args = ["--arg1", "--arg2"]
env = { API_KEY = "test123" }

[mcp.opencode.analyzer]
command = "analyzer"
"#;
        let config: KlothoConfig = toml::from_str(toml_content).unwrap();
        let mcp = config.mcp.unwrap();

        // Check shared servers
        assert_eq!(mcp.servers.len(), 1);
        let context7 = mcp.servers.get("context7").unwrap();
        assert_eq!(context7.command, "uvx");
        assert_eq!(context7.args, vec!["@upstash/context7-mcp"]);

        // Check claude-specific
        assert_eq!(mcp.claude.len(), 1);
        let custom = mcp.claude.get("custom-tool").unwrap();
        assert_eq!(custom.command, "/usr/bin/custom");
        assert_eq!(custom.args, vec!["--arg1", "--arg2"]);
        assert_eq!(custom.env.get("API_KEY"), Some(&"test123".to_string()));

        // Check opencode-specific
        assert_eq!(mcp.opencode.len(), 1);
        let analyzer = mcp.opencode.get("analyzer").unwrap();
        assert_eq!(analyzer.command, "analyzer");
    }

    #[test]
    fn test_volumespec_resolve_simple() {
        let vol = VolumeSpec::Simple("/path/to/data".to_string());
        let (src, dest, readonly) = vol.resolve();
        assert_eq!(src, "/path/to/data");
        assert_eq!(dest, "/path/to/data");
        assert!(!readonly);
    }

    #[test]
    fn test_volumespec_resolve_detailed() {
        let vol = VolumeSpec::Detailed {
            src: "/host/path".to_string(),
            dest: "/container/path".to_string(),
            readonly: true,
        };
        let (src, dest, readonly) = vol.resolve();
        assert_eq!(src, "/host/path");
        assert_eq!(dest, "/container/path");
        assert!(readonly);
    }

    #[test]
    fn test_volumespec_resolve_tilde_expansion() {
        std::env::set_var("HOME", "/home/testuser");

        let vol = VolumeSpec::Simple("~/mydata".to_string());
        let (src, dest, _) = vol.resolve();
        assert_eq!(src, "/home/testuser/mydata");
        assert_eq!(dest, "/home/testuser/mydata");

        let vol2 = VolumeSpec::Detailed {
            src: "~/host/data".to_string(),
            dest: "~/container/data".to_string(),
            readonly: false,
        };
        let (src2, dest2, _) = vol2.resolve();
        assert_eq!(src2, "/home/testuser/host/data");
        assert_eq!(dest2, "/home/testuser/container/data");
    }

    #[test]
    fn test_parse_complete_klotho_toml() {
        let toml_content = r#"
[project]
agent = "claude"
name = "test-project"

[packages.apt]
gcc = "*"

[packages.pip]
pytest = ">=7.0"

[[volumes]]
src = "~/data"
dest = "/work/data"
readonly = true

[mcp.servers.shared]
command = "shared-server"
args = ["--port", "8080"]
"#;
        let config: KlothoConfig = toml::from_str(toml_content).unwrap();

        // Verify all sections parsed correctly
        assert!(config.project.is_some());
        assert!(config.packages.is_some());
        assert_eq!(config.volumes.len(), 1);
        assert!(config.mcp.is_some());

        let project = config.project.unwrap();
        assert_eq!(project.agent, Some("claude".to_string()));

        let packages = config.packages.unwrap();
        assert_eq!(packages.apt.get("gcc"), Some(&"*".to_string()));

        let mcp = config.mcp.unwrap();
        assert!(mcp.servers.contains_key("shared"));
    }

    #[test]
    fn test_mcp_to_claude_json() {
        let mut servers = HashMap::new();
        servers.insert(
            "test-server".to_string(),
            McpServerConfig {
                command: "test-cmd".to_string(),
                args: vec!["arg1".to_string(), "arg2".to_string()],
                env: [("KEY".to_string(), "value".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            },
        );

        let json = mcp_to_claude_json(&servers);

        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("mcpServers"));

        let mcp_servers = obj.get("mcpServers").unwrap().as_object().unwrap();
        assert!(mcp_servers.contains_key("test-server"));

        let server = mcp_servers.get("test-server").unwrap().as_object().unwrap();
        assert_eq!(server.get("command").unwrap().as_str().unwrap(), "test-cmd");

        let args = server.get("args").unwrap().as_array().unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].as_str().unwrap(), "arg1");

        let env = server.get("env").unwrap().as_object().unwrap();
        assert_eq!(env.get("KEY").unwrap().as_str().unwrap(), "value");
    }

    #[test]
    fn test_mcp_to_opencode_json() {
        let mut servers = HashMap::new();
        servers.insert(
            "test-server".to_string(),
            McpServerConfig {
                command: "test-cmd".to_string(),
                args: vec!["arg1".to_string()],
                env: HashMap::new(),
            },
        );

        let json = mcp_to_opencode_json(&servers);

        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("mcp"));

        let mcp = obj.get("mcp").unwrap().as_object().unwrap();
        assert!(mcp.contains_key("test-server"));

        let server = mcp.get("test-server").unwrap().as_object().unwrap();
        assert_eq!(server.get("type").unwrap().as_str().unwrap(), "local");
        assert_eq!(server.get("enabled").unwrap().as_bool().unwrap(), true);

        let command = server.get("command").unwrap().as_array().unwrap();
        assert_eq!(command.len(), 2);
        assert_eq!(command[0].as_str().unwrap(), "test-cmd");
        assert_eq!(command[1].as_str().unwrap(), "arg1");
    }

    #[test]
    fn test_resolve_mcp_servers_agent_specific() {
        let mut mcp = McpConfig::default();
        mcp.servers.insert(
            "shared".to_string(),
            McpServerConfig {
                command: "shared-cmd".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
        );
        mcp.claude.insert(
            "claude-specific".to_string(),
            McpServerConfig {
                command: "claude-cmd".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
        );

        // When agent-specific exists, it replaces shared
        let resolved = resolve_mcp_servers(&mcp, "claude");
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key("claude-specific"));
        assert!(!resolved.contains_key("shared"));
    }

    #[test]
    fn test_resolve_mcp_servers_fallback_to_shared() {
        let mut mcp = McpConfig::default();
        mcp.servers.insert(
            "shared".to_string(),
            McpServerConfig {
                command: "shared-cmd".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
        );

        // When no agent-specific exists, use shared
        let resolved = resolve_mcp_servers(&mcp, "claude");
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key("shared"));

        // Same for opencode
        let resolved = resolve_mcp_servers(&mcp, "opencode");
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key("shared"));
    }

    #[test]
    fn test_resolve_env_vars_with_valid_var() {
        std::env::set_var("TEST_API_KEY", "sk-test-12345");

        let result = resolve_env_vars("${TEST_API_KEY}").unwrap();
        assert_eq!(result, "sk-test-12345");

        // Mixed content
        let result = resolve_env_vars("prefix-${TEST_API_KEY}-suffix").unwrap();
        assert_eq!(result, "prefix-sk-test-12345-suffix");

        std::env::remove_var("TEST_API_KEY");
    }

    #[test]
    fn test_resolve_env_vars_with_missing_var() {
        // Ensure the var doesn't exist
        std::env::remove_var("MISSING_VAR_12345");

        let result = resolve_env_vars("${MISSING_VAR_12345}");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("MISSING_VAR_12345"));
        assert!(err.contains("is not set"));
    }

    #[test]
    fn test_resolve_env_vars_no_vars() {
        let result = resolve_env_vars("plain-string-no-vars").unwrap();
        assert_eq!(result, "plain-string-no-vars");
    }

    #[test]
    fn test_resolve_credentials() {
        std::env::set_var("TEST_CRED_KEY", "resolved-key");

        let creds = AgentCredentials {
            api_key: Some("${TEST_CRED_KEY}".to_string()),
        };

        let resolved = resolve_credentials(&creds).unwrap();
        assert_eq!(resolved.api_key, Some("resolved-key".to_string()));

        std::env::remove_var("TEST_CRED_KEY");
    }

    #[test]
    fn test_resolve_credentials_none() {
        let creds = AgentCredentials { api_key: None };

        let resolved = resolve_credentials(&creds).unwrap();
        assert_eq!(resolved.api_key, None);
    }

    #[test]
    fn test_parse_agents_section() {
        let toml_content = r#"
[agents.claude]
api_key = "sk-test-key"

[agents.opencode]
api_key = "${OPENAI_API_KEY}"
"#;
        let config: KlothoConfig = toml::from_str(toml_content).unwrap();

        assert_eq!(config.agents.len(), 2);
        assert_eq!(
            config.agents.get("claude").unwrap().api_key,
            Some("sk-test-key".to_string())
        );
        assert_eq!(
            config.agents.get("opencode").unwrap().api_key,
            Some("${OPENAI_API_KEY}".to_string())
        );
    }

    #[test]
    fn test_parse_mount_host_config() {
        let toml_content = r#"
mount_host_config = true
"#;
        let config: KlothoConfig = toml::from_str(toml_content).unwrap();
        assert!(config.mount_host_config);

        // Default is false
        let config2: KlothoConfig = toml::from_str("").unwrap();
        assert!(!config2.mount_host_config);
    }
}
