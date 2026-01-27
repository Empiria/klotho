---
status: resolved
trigger: "klotho start -a opencode -n infinite-worlds fails with could not find config/agents directory"
created: 2026-01-27T12:00:00Z
updated: 2026-01-27T12:05:00Z
---

## Current Focus

hypothesis: CONFIRMED - Root cause identified and fixed
test: N/A - fix verified
expecting: Embedded configs allow standalone binary to work
next_action: Archive session

## Symptoms

expected: klotho start command should start successfully and launch the opencode agent
actual: Error: could not find config/agents directory - searched relative to executable and current directory
errors: "Error: could not find config/agents directory / searched relative to executable and current directory"
reproduction: Run `klotho start -a opencode -n infinite-worlds` from a directory that is NOT the klotho repo or project root
started: Recently stopped working - used to work before

## Eliminated

## Evidence

- timestamp: 2026-01-27T12:00:30Z
  checked: src/config.rs get_repo_config_dir() function
  found: |
    Function searches for config/agents in these paths:
    1. Same directory as executable (/home/owen/.local/bin/)
    2. Parent of executable (/home/owen/.local/)
    3. Two levels up (/home/owen/)
    4. Current working directory
    5. Git repository root of current directory
  implication: When running from another project, git root is wrong project, and installed binary is standalone

- timestamp: 2026-01-27T12:00:45Z
  checked: Installed binary location
  found: /home/owen/.local/bin/klotho is a regular file (not symlink), 1.8MB
  implication: Binary is standalone, not linked to repo - config/agents is not nearby

- timestamp: 2026-01-27T12:01:00Z
  checked: config/agents in klotho repo
  found: /home/owen/projects/personal/klotho/config/agents/ exists with claude/ and opencode/ subdirs
  implication: Config exists in repo but not findable when running installed binary from other locations

- timestamp: 2026-01-27T12:02:00Z
  checked: src/resources.rs module
  found: Already has embedded agent configs via rust_embed, with get_agent_config() and list_embedded_agents()
  implication: Fix is simpler - just use existing resources module instead of broken get_repo_config_dir()

- timestamp: 2026-01-27T12:04:00Z
  checked: Manual test of fix
  found: |
    Running `klotho start -a opencode -n test-session` from /tmp succeeds:
    - "Creating new session 'test-session'..."
    - "Created session 'test-session' -> klotho-session-opencode-test-session"
    - Zellij and OpenCode launched successfully
  implication: Fix verified - embedded configs work correctly

## Resolution

root_cause: |
  The klotho binary installed to ~/.local/bin/ cannot find config/agents because:
  1. load_agent_config() in config.rs called get_repo_config_dir() which searches filesystem
  2. Search paths only look relative to executable, cwd, and git root
  3. When running from another project, none of these paths contain config/agents
  4. The resources module already had embedded configs but config.rs wasn't using them

fix: |
  Modified load_agent_config() in src/config.rs to:
  1. Use resources::get_agent_config() for embedded defaults (always available)
  2. Still layer user configs from ~/.config/klotho/agents/ on top
  3. Removed unused get_repo_config_dir() function

verification: |
  1. cargo build --release: SUCCESS
  2. cargo test: 16 tests passed
  3. Manual test from /tmp: klotho start -a opencode -n test-session works
  4. Installed updated binary to ~/.local/bin/klotho

files_changed:
  - src/config.rs: Use embedded configs, remove get_repo_config_dir()
