use anyhow::{Context, Result, bail};
use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
        readonly: bool
    },
    /// Simple mount specification as a single string (e.g., "/host/path:/container/path")
    Simple(String),
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct KlothoConfig {
    #[serde(default)]
    pub packages: Option<Packages>,
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

    toml::from_str(&contents)
        .context(format!("Failed to parse {} as TOML", config_path.display()))
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

        commands.push(format!(
            "RUN npm install -g {}",
            pkg_specs.join(" ")
        ));
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

        merge_cli_installs(&mut packages, &[
            "apt:gcc".to_string(),
            "pip:pytest==7.0".to_string(),
            "npm:typescript=^5.0".to_string(),
            "cargo:ripgrep".to_string(),
        ])
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
        assert!(result.unwrap_err().to_string().contains("Unknown package manager"));
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

        packages.npm.insert("bad;package".to_string(), "*".to_string());
        assert!(validate_all_packages(&packages).is_err());
    }

    #[test]
    fn test_generate_apt_commands() {
        let mut packages = Packages::default();
        packages.apt.insert("gcc".to_string(), "*".to_string());
        packages.apt.insert("python3".to_string(), "3.11".to_string());

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
        packages.pip.insert("pytest".to_string(), ">=7.0".to_string());
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
        packages.npm.insert("typescript".to_string(), "^5.0".to_string());
        packages.npm.insert("@types/node".to_string(), "*".to_string());

        let commands = generate_install_commands(&packages);

        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("npm install -g"));
        assert!(commands[0].contains("typescript@^5.0"));
        assert!(commands[0].contains("@types/node"));
    }

    #[test]
    fn test_generate_cargo_commands() {
        let mut packages = Packages::default();
        packages.cargo.insert("ripgrep".to_string(), "*".to_string());
        packages.cargo.insert("fd-find".to_string(), "8.7.0".to_string());

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
        packages.cargo.insert("ripgrep".to_string(), "*".to_string());

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
        packages.npm.insert("typescript".to_string(), "*".to_string());

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
        packages.npm.insert("typescript".to_string(), "*".to_string());
        packages.cargo.insert("ripgrep".to_string(), "*".to_string());

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
}
