# ===== BASE STAGE (shared by all agents) =====
FROM --platform=linux/amd64 debian:bookworm-slim AS base

# Install common tools
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

# Create non-root user with fish (UID 1000 to match typical host user)
RUN useradd -m -s /usr/bin/fish -u 1000 agent

# Entrypoint handles config setup
COPY --chmod=755 entrypoint.sh /entrypoint.sh

# Switch to agent user for remaining setup
USER agent

# Ensure ~/.local/bin exists (native installers use this)
# Also create ~/.local/share/fish for history storage
RUN mkdir -p ~/.local/bin ~/.local/share/fish

# Configure fish: disable greeting, enable starship prompt
RUN mkdir -p ~/.config/fish && printf '%s\n' \
    'set -g fish_greeting' \
    'starship init fish | source' \
    > ~/.config/fish/config.fish

# Set common environment
ENV PATH="/home/agent/.local/bin:$PATH"

WORKDIR /workspace
ENTRYPOINT ["/entrypoint.sh"]
CMD ["zellij"]

# ===== CLAUDE AGENT STAGE =====
FROM base AS claude

# Accept build args for config values
ARG AGENT_NAME
ARG AGENT_INSTALL_CMD
ARG AGENT_SHELL
ARG AGENT_LAUNCH_CMD

# Install uv (provides uvx for Python MCP servers)
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# Install Claude Code using config value
RUN eval "$AGENT_INSTALL_CMD"

# Create agent wrapper script using AGENT_LAUNCH_CMD
RUN printf '%s\n' \
    '#!/usr/bin/env fish' \
    "$AGENT_LAUNCH_CMD" \
    'exec fish' \
    > ~/.local/bin/${AGENT_NAME}-session && chmod +x ~/.local/bin/${AGENT_NAME}-session

# Set environment from config
ENV SHELL="$AGENT_SHELL"

# ===== OPENCODE AGENT STAGE =====
FROM base AS opencode

# Accept build args for config values
ARG AGENT_NAME
ARG AGENT_INSTALL_CMD
ARG AGENT_SHELL
ARG AGENT_LAUNCH_CMD

# Install uv (provides uvx for Python MCP servers)
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# Install OpenCode using config value
RUN eval "$AGENT_INSTALL_CMD"

# Create agent wrapper script using AGENT_LAUNCH_CMD
RUN printf '%s\n' \
    '#!/usr/bin/env fish' \
    "$AGENT_LAUNCH_CMD" \
    'exec fish' \
    > ~/.local/bin/${AGENT_NAME}-session && chmod +x ~/.local/bin/${AGENT_NAME}-session

# Set environment from config
ENV SHELL="$AGENT_SHELL"
