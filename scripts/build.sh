#!/bin/bash
# Build script for Klotho agent container images
# Usage: ./scripts/build.sh <agent-name>
set -euo pipefail

AGENT_NAME="${1:-}"

# Show usage if no agent name provided
if [[ -z "$AGENT_NAME" ]]; then
    echo "Usage: $0 <agent-name>" >&2
    echo "" >&2
    echo "Example: $0 claude" >&2
    exit 1
fi

# Validate config file format - reject command substitution
validate_config() {
    local config_file="$1"

    # Check file exists and is readable
    if [[ ! -f "$config_file" ]] || [[ ! -r "$config_file" ]]; then
        echo "Error: Config file not found or not readable: $config_file" >&2
        return 1
    fi

    # Reject command substitution which would execute during sourcing
    # Backticks and $() are dangerous even inside double quotes
    # Note: Pipes, semicolons, ampersands inside quoted strings are safe
    if grep -qE '`|\$\(' "$config_file"; then
        echo "Error: Config contains command substitution (\$() or backticks)" >&2
        echo "Config files may only contain KEY=value pairs and variable expansion (\$VAR)" >&2
        return 1
    fi

    return 0
}

# Load agent config with XDG-style layering
load_agent_config() {
    local agent="$1"
    local config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
    # Check klotho config first (preferred), then legacy agent-session
    local user_config=""
    if [[ -f "$config_home/klotho/agents/$agent/config.conf" ]]; then
        user_config="$config_home/klotho/agents/$agent/config.conf"
    elif [[ -f "$config_home/agent-session/agents/$agent/config.conf" ]]; then
        user_config="$config_home/agent-session/agents/$agent/config.conf"
        echo "Note: Using legacy config path ~/.config/agent-session/"
        echo "      Consider moving to ~/.config/klotho/"
    fi
    local repo_config="./config/agents/$agent/config.conf"

    # Load repo default first (must exist)
    if [[ ! -f "$repo_config" ]]; then
        echo "Error: No default config found for agent: $agent" >&2
        echo "Expected: $repo_config" >&2
        return 1
    fi

    validate_config "$repo_config" || return 1
    source "$repo_config"

    # Override with user config if present
    if [[ -n "$user_config" && -f "$user_config" ]]; then
        echo "Loading user config override: $user_config"
        validate_config "$user_config" || return 1
        source "$user_config"
    fi

    return 0
}

# Validate all required config fields are non-empty
validate_required_fields() {
    local required_fields=(
        "AGENT_NAME"
        "AGENT_DESCRIPTION"
        "AGENT_INSTALL_CMD"
        "AGENT_LAUNCH_CMD"
        "AGENT_SHELL"
    )

    local missing=()

    for field in "${required_fields[@]}"; do
        if [[ -z "${!field:-}" ]]; then
            missing+=("$field")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        echo "Error: Missing required config fields:" >&2
        printf '  - %s\n' "${missing[@]}" >&2
        return 1
    fi

    return 0
}

# Validate that the Containerfile has a matching stage
validate_stage_exists() {
    local containerfile="Containerfile"
    local stage_name="$1"

    # Check if "FROM ... AS <stage_name>" exists in Containerfile
    if ! grep -qE "^FROM .* AS ${stage_name}\$" "$containerfile"; then
        echo "Error: Stage '$stage_name' not found in $containerfile" >&2
        echo "" >&2
        echo "Available stages:" >&2
        grep -E '^FROM .* AS ' "$containerfile" | sed 's/^FROM .* AS /  - /' >&2
        return 1
    fi

    return 0
}

# Main build process
main() {
    echo "Building agent: $AGENT_NAME"
    echo ""

    # Load and validate config
    load_agent_config "$AGENT_NAME" || exit 1
    validate_required_fields || exit 1

    # Validate stage exists in Containerfile
    validate_stage_exists "$AGENT_NAME" || exit 1

    # Build with config injection
    echo ""
    echo "Building container image with target: $AGENT_NAME"
    echo "Image tag: klotho-${AGENT_NAME}:latest"
    echo ""

    podman build \
        --target="$AGENT_NAME" \
        --build-arg AGENT_NAME="$AGENT_NAME" \
        --build-arg AGENT_INSTALL_CMD="$AGENT_INSTALL_CMD" \
        --build-arg AGENT_SHELL="$AGENT_SHELL" \
        --build-arg AGENT_LAUNCH_CMD="$AGENT_LAUNCH_CMD" \
        -t "klotho-${AGENT_NAME}:latest" \
        .

    echo ""
    echo "Build complete: klotho-${AGENT_NAME}:latest"
}

main
