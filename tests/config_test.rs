use klotho::config;

#[test]
fn test_load_claude_config() {
    // This test verifies config loading from the repo's config directory
    match config::load_agent_config("claude") {
        Ok(agent_config) => {
            assert_eq!(agent_config.name, "claude");
            assert!(!agent_config.description.is_empty());
            assert!(!agent_config.launch_cmd.is_empty());
            assert!(!agent_config.shell.is_empty());
            assert!(!agent_config.env_vars.is_empty());
        }
        Err(e) => {
            panic!("Failed to load claude config: {}", e);
        }
    }
}

#[test]
fn test_toml_deserialization() {
    use klotho::agent::AgentConfig;

    // Test that TOML config deserializes correctly
    let config_toml = r#"
name = "test"
description = "Test agent"
shell = "/bin/bash"
install_cmd = "echo install"
launch_cmd = "echo launch"
env_vars = ["PATH=/usr/bin", "SHELL=/bin/bash"]
"#;

    let result: Result<AgentConfig, _> = toml::from_str(config_toml);
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.name, "test");
    assert_eq!(config.env_vars.len(), 2);
}
