#!/bin/bash
set -e

# Save original working directory
WORKDIR="$(pwd)"

# Set up ~/.claude from mounted /config/.claude
# Symlink valid items, create directories for broken symlinks
if [[ -d /config/.claude && ! -d ~/.claude ]]; then
    mkdir -p ~/.claude
    for item in /config/.claude/* /config/.claude/.*; do
        [[ "$(basename "$item")" == "." || "$(basename "$item")" == ".." ]] && continue
        [[ ! -e "$item" && ! -L "$item" ]] && continue

        name=$(basename "$item")
        if [[ -L "$item" && ! -e "$item" ]]; then
            # Broken symlink - create directory instead
            mkdir -p ~/.claude/"$name"
        else
            # Valid file/dir/symlink - symlink to mounted version
            ln -s /config/.claude/"$name" ~/.claude/"$name"
        fi
    done
fi

# Copy mounted configs to home directory (allows writes, fixes permissions)
[[ -d /config/zellij ]] && mkdir -p ~/.config && cp -r /config/zellij ~/.config/

# Install GSD plugin if not present
if [[ ! -f ~/.claude/get-shit-done/VERSION ]]; then
    echo "Installing get-shit-done..."
    npx -y get-shit-done-cc@latest --claude --global
fi

echo "Claude Code $(claude --version) ready"

# Restore working directory and exec
cd "$WORKDIR"
exec "$@"
