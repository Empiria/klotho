FROM --platform=linux/amd64 debian:bookworm-slim

# Install essentials
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    git \
    ca-certificates \
    fish \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install Zellij (stable, rarely changes)
RUN curl -sSL https://github.com/zellij-org/zellij/releases/latest/download/zellij-x86_64-unknown-linux-musl.tar.gz \
    | tar xz -C /usr/local/bin

# Install Starship prompt
RUN curl -sSL https://starship.rs/install.sh | sh -s -- -y

# Create non-root user with zsh (UID 1000 to match typical host user)
RUN useradd -m -s /usr/bin/fish -u 1000 agent

# Entrypoint handles config setup
COPY --chmod=755 entrypoint.sh /entrypoint.sh

# Install tools as agent user
USER agent

# Ensure ~/.local/bin exists (native installers use this)
RUN mkdir -p ~/.local/bin

# Configure fish: disable greeting, enable starship prompt
RUN mkdir -p ~/.config/fish && printf '%s\n' \
    'set -g fish_greeting' \
    'starship init fish | source' \
    > ~/.config/fish/config.fish

# Create claude wrapper script (starts claude, drops to fish on exit)
RUN printf '%s\n' \
    '#!/usr/bin/env fish' \
    'claude --dangerously-skip-permissions' \
    'exec fish' \
    > ~/.local/bin/claude-session && chmod +x ~/.local/bin/claude-session

# Install uv (provides uvx for Python MCP servers)
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# Install Claude Code using native installer
RUN curl -fsSL https://claude.ai/install.sh | bash

ENV PATH="/home/agent/.local/bin:$PATH"
ENV SHELL="/usr/bin/fish"

WORKDIR /workspace
ENTRYPOINT ["/entrypoint.sh"]
CMD ["zellij"]
