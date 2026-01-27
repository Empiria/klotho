use klotho::config;

#[test]
fn test_load_claude_config() {
    // This test verifies config loading from the repo's config directory
    match config::load_agent_config("claude") {
        Ok((agent_config, _is_legacy)) => {
            assert_eq!(agent_config.name, "claude");
            assert!(!agent_config.description.is_empty());
            assert!(!agent_config.launch_cmd.is_empty());
            assert!(!agent_config.shell.is_empty());
        }
        Err(e) => {
            panic!("Failed to load claude config: {}", e);
        }
    }
}

#[test]
fn test_config_security_validation() {
    use klotho::agent::AgentConfig;

    // Test that command substitution is rejected
    let bad_config = r#"
AGENT_NAME="bad"
AGENT_SHELL="$(whoami)"
"#;

    let result = AgentConfig::from_keyvalue(bad_config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("command substitution"));
}
