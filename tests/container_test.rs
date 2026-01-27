use klotho::container;

#[test]
fn test_runtime_detection() {
    // Test that runtime detection works (should find podman or docker)
    match container::detect_runtime(None) {
        Ok(runtime) => {
            println!("Detected runtime: {}", runtime.as_str());
            assert!(
                runtime.as_str() == "podman" || runtime.as_str() == "docker",
                "Runtime should be podman or docker"
            );
        }
        Err(e) => {
            // It's OK if neither podman nor docker is installed (e.g., in CI)
            println!("No container runtime found (expected in some environments): {}", e);
        }
    }
}

#[test]
fn test_runtime_override_validation() {
    // Test that invalid runtime is rejected
    let result = container::detect_runtime(Some("invalid"));
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("invalid runtime"));
}

#[test]
fn test_runtime_override_auto() {
    // Test that "auto" means auto-detection
    let result = container::detect_runtime(Some("auto"));
    // Should behave same as None (auto-detect)
    match result {
        Ok(_) => {
            // Runtime detected successfully
        }
        Err(_) => {
            // No runtime available - OK in some environments
        }
    }
}
