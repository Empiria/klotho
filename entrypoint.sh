#!/bin/bash
set -e

# Save original working directory
WORKDIR="$(pwd)"

# Set up ~/.claude from mounted /config/.claude
# Symlink valid items, create directories for broken symlinks
# Note: ~/.claude may already exist from image build, so we merge rather than replace
if [[ -d /config/.claude ]]; then
    mkdir -p ~/.claude
    for item in /config/.claude/* /config/.claude/.*; do
        [[ "$(basename "$item")" == "." || "$(basename "$item")" == ".." ]] && continue
        [[ ! -e "$item" && ! -L "$item" ]] && continue

        name=$(basename "$item")
        target=~/.claude/"$name"

        # Skip if target already exists (prefer container version for runtime dirs)
        [[ -e "$target" || -L "$target" ]] && continue

        if [[ -L "$item" && ! -e "$item" ]]; then
            # Broken symlink - create directory instead
            mkdir -p "$target"
        else
            # Valid file/dir/symlink - symlink to mounted version
            ln -s /config/.claude/"$name" "$target"
        fi
    done
fi

# Set up ~/.config/opencode from mounted /config/opencode
# Similar pattern to Claude: merge mounted config with any existing container config
if [[ -d /config/opencode ]]; then
    mkdir -p ~/.config/opencode
    for item in /config/opencode/* /config/opencode/.*; do
        [[ "$(basename "$item")" == "." || "$(basename "$item")" == ".." ]] && continue
        [[ ! -e "$item" && ! -L "$item" ]] && continue

        name=$(basename "$item")
        target=~/.config/opencode/"$name"

        # Skip if target already exists (prefer container version)
        [[ -e "$target" || -L "$target" ]] && continue

        if [[ -L "$item" && ! -e "$item" ]]; then
            # Broken symlink - create directory instead
            mkdir -p "$target"
        else
            # Valid file/dir/symlink - symlink to mounted version
            ln -s /config/opencode/"$name" "$target"
        fi
    done
fi

# Copy mounted configs to home directory (allows writes, fixes permissions)
[[ -d /config/zellij ]] && mkdir -p ~/.config && cp -r /config/zellij ~/.config/

# Install skills from config (passed as JSON in KLOTHO_SKILLS env var)
if [[ -n "${KLOTHO_SKILLS:-}" ]]; then
    echo "Installing skills..."
    
    # Parse JSON and install each skill using jq
    echo "$KLOTHO_SKILLS" | jq -r 'to_entries[] | @base64' | while read -r skill_b64; do
        skill=$(echo "$skill_b64" | base64 -d)
        name=$(echo "$skill" | jq -r '.key')
        install_cmd=$(echo "$skill" | jq -r '.value.install')
        setup_cmd=$(echo "$skill" | jq -r '.value.setup // empty')
        
        echo "  Installing skill: $name"
        
        # Run install command
        if ! eval "$install_cmd"; then
            echo "ERROR: Failed to install skill '$name'"
            exit 1
        fi
        
        # Run setup command if provided
        if [[ -n "$setup_cmd" ]]; then
            if ! eval "$setup_cmd"; then
                echo "ERROR: Failed to run setup for skill '$name'"
                exit 1
            fi
        fi
        
        echo "  Skill '$name' installed successfully"
    done
fi

# Print agent version if available
if command -v claude &>/dev/null; then
    echo "Claude Code $(claude --version) ready"
elif command -v opencode &>/dev/null; then
    echo "OpenCode ready"
    echo "Run /connect to configure API keys if not already set up"
else
    echo "Agent environment ready"
fi

# Restore working directory and exec
cd "$WORKDIR"
exec "$@"
