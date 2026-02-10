use anyhow::{Context, Result};
use std::path::Path;

const TEMPLATE_CONTENT: &str = r#"# .klotho.toml - klotho container configuration
# Packages specified here are installed during `klotho build`
# and available in all sessions for this project.
#
# Run `klotho build` after editing to apply changes.

# [packages.apt]
# # System packages (Debian/Ubuntu)
# # Use "*" for latest version
# gcc = "*"
# build-essential = "*"

# [packages.pip]
# # Python packages (pip install)
# pytest = ">=7.0"

# [packages.npm]
# # Node.js packages (npm install -g)
# typescript = "^5.0"

# [packages.cargo]
# # Rust packages (cargo install)
# ripgrep = "*"
#
# # Known runtimes (use these instead of manual setup):
# # rustup = "*"   → installs Rust toolchain via rustup
# # nvm = "*"      → installs Node.js via nvm (in [packages.npm])
"#;

/// Initialize a .klotho.toml file in the current directory
pub fn run() -> Result<()> {
    let config_path = Path::new(".klotho.toml");

    // Check if file already exists
    if config_path.exists() {
        anyhow::bail!(".klotho.toml already exists");
    }

    // Write template
    std::fs::write(config_path, TEMPLATE_CONTENT)
        .context("Failed to write .klotho.toml")?;

    eprintln!("Created .klotho.toml - edit to add packages, then run `klotho build`");

    Ok(())
}
