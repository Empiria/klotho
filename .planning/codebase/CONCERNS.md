# Codebase Concerns

**Analysis Date:** 2026-01-26

## Security Considerations

**Unverified installer scripts downloaded via curl and piped to shell:**
- Risk: Man-in-the-middle attack, compromised installer, unsigned downloads
- Files: `Containerfile` lines 18, 46, 49
- Current mitigation: Uses HTTPS and `curl -sSL` with checksum verification unavailable for most
- Recommendations:
  - Implement checksum verification for Zellij (validate SHA256 after download)
  - Pin specific versions instead of `/latest/` for Zellij releases
  - Use `curl -fsSL https://starship.rs/install.sh | bash` - requires trust; consider pinning version or downloading pre-built binary
  - For uv and Claude Code installers, verify checksums or use package managers if available
  - Document the security implications in README

**Unquoted variables in shell expansion:**
- Risk: Word splitting and glob expansion if variables contain spaces or special characters
- Files: `agent-session` line 77, 120, 130, 147-148
- Current mitigation: Paths are validated and made absolute with `realpath`
- Recommendations: Quote all variable expansions: `"$MOUNTS"`, `"$EXTRA_MOUNTS"`, `$MOUNT_PATHS[@]`

**Session name injection risk:**
- Risk: SESSION_NAME is user-controlled and used in container name and zellij session
- Files: `agent-session` lines 56, 72, 81, 86, 93
- Current mitigation: Container name uses safe prefix format; zellij/podman should validate
- Recommendations: Whitelist session names to alphanumeric + dash: `[[ ! $SESSION_NAME =~ ^[a-zA-Z0-9-]+$ ]] && exit 1`

**Sensitive file mounts without permission enforcement:**
- Risk: ~/.claude.json, ~/.local/share/claude mounted RW; could contain API keys or credentials
- Files: `agent-session` lines 143-145
- Current mitigation: User-initiated mounts; container isolation provides boundary
- Recommendations: Mount `.claude.json` as read-only if possible; document sensitivity; audit what gets stored there

**npx without version pinning:**
- Risk: `npx -y get-shit-done-cc@latest` downloads and executes latest version without verification
- Files: `entrypoint.sh` line 32
- Current mitigation: `@latest` tag ensures current version
- Recommendations: Pin to specific version: `npx -y get-shit-done-cc@1.x.x` to prevent breaking changes

**claude --dangerously-skip-permissions:**
- Risk: Intentionally disables permission checks; could allow unsafe operations
- Files: `Containerfile` line 41, documented in `agent-session` line 30
- Current mitigation: Runs in isolated container with user identity
- Recommendations: Document the security model; clarify when this flag is necessary

## Error Handling Gaps

**No error checking on symlink creation failures:**
- Problem: `ln -s` can fail silently if symlink already exists as wrong type
- Files: `entrypoint.sh` lines 21
- Impact: Config loading could be incomplete without visible error
- Fix approach: Add error check: `ln -s ... || mkdir -p ~/.claude/"$name"`

**No validation that realpath succeeds:**
- Problem: If path resolution fails, MOUNTS could be empty or malformed
- Files: `agent-session` line 118
- Impact: Container startup might succeed with wrong or missing mounts
- Fix approach: Add error check after realpath: `[[ -z "$abs_path" ]] && { echo "error: realpath failed for $path"; exit 1; }`

**Sleep-based synchronization is unreliable:**
- Problem: Hard-coded `sleep 1` assumes container/service readiness
- Files: `agent-session` lines 96, 153
- Impact: Race condition - services might not be ready after 1 second
- Fix approach: Implement health checks or wait loops that verify container readiness

**No handling of zellij session attachment failures:**
- Problem: attach_zellij function has no error handling if podman exec fails
- Files: `agent-session` lines 78, 81
- Impact: Script exits silently, user sees no error message
- Fix approach: Remove `2>/dev/null` redirection, add explicit error handling for podman exec failures

**Missing cleanup on container creation failure:**
- Problem: If `podman run` fails, no cleanup; orphaned container possible if partial creation succeeds
- Files: `agent-session` lines 139-150
- Impact: User must manually clean up failed containers
- Fix approach: Add trap handler: `trap 'podman rm -f $CONTAINER_NAME' ERR`

## Performance Bottlenecks

**Fixed 1-second sleep delays create latency:**
- Problem: Both container startup and zellij session checks use hard-coded sleep
- Files: `agent-session` lines 96, 153
- Impact: Every session attachment adds 1-2 seconds overhead
- Improvement path: Implement exponential backoff or health endpoint polling instead of sleep

**Full container image rebuild on every change:**
- Problem: Containerfile installs tools on every build; no layer caching strategy
- Files: `Containerfile` lines 14, 18, 46, 49
- Impact: Cold builds take 2-5 minutes due to downloading and installing each tool
- Improvement path: Separate tools by change frequency - pin stable tools, update less frequently. Use multi-stage builds.

**Directory listing with ANSI stripping on every attach:**
- Problem: `zellij list-sessions` output is parsed with sed regex on every session check
- Files: `agent-session` line 77
- Impact: Minimal but unnecessary overhead
- Improvement path: Use `zellij list-sessions --no-color` if available instead of sed parsing

## Fragile Areas

**entrypoint.sh broken symlink handling:**
- Files: `entrypoint.sh` lines 9-24
- Why fragile: Complex glob logic with `.` and `..` filtering, broken symlink detection
- Safe modification: Test glob patterns thoroughly; consider using `find /config/.claude` instead of glob
- Test coverage: No test for missing /config/.claude directory or permission denied scenarios

**agent-session mount path parsing:**
- Files: `agent-session` lines 117-122
- Why fragile: Path basename used as container dir name; collision risk if multiple projects have same basename
- Safe modification: Add uniqueness checking or use full path hash for directory names
- Test coverage: No test for paths with spaces, special chars, or basename collisions

**Podman availability assumption:**
- Files: `agent-session` throughout, especially lines 86, 93, 95, 139
- Why fragile: Script assumes `podman` is installed and accessible; no version checking
- Safe modification: Add early check: `command -v podman >/dev/null || { echo "error: podman not found"; exit 1; }`
- Test coverage: No test for podman absence or old version

**Zellij session list parsing with regex:**
- Files: `agent-session` line 77
- Why fragile: Regex `"^$SESSION_NAME "` assumes specific format; ANSI stripping uses hardcoded pattern `\x1b\[[0-9;]*m`
- Safe modification: Use `zellij --plain` or similar if available; document exact format expected
- Test coverage: No test for unusual session names or zellij output variations

## Test Coverage Gaps

**No integration tests for container creation:**
- What's not tested: Multiple paths mounting, path validation, mount ordering
- Files: `agent-session` lines 114-150
- Risk: Wrong mounts or missing mounts could go undetected
- Priority: High - this is the core functionality

**No tests for broken container recovery:**
- What's not tested: Reattaching to stopped containers, container state transitions
- Files: `agent-session` lines 85-98
- Risk: Container state mismatch could cause silent failures or orphaned processes
- Priority: Medium - affects reliability

**No tests for session name edge cases:**
- What's not tested: Special characters, spaces, very long names, Unicode
- Files: `agent-session` lines 56, 72, 81
- Risk: Session creation could fail or create containers with invalid names
- Priority: Medium - security and usability

**No tests for AGENT_SESSION_MOUNTS parsing:**
- What's not tested: Colon-separated paths, missing paths, permission denied
- Files: `agent-session` lines 126-134
- Risk: Malformed mounts silently skipped with warning only
- Priority: Medium - silent failures reduce debuggability

**No tests for environment variable handling:**
- What's not tested: HOME not set, invalid paths, permission issues
- Files: `agent-session` lines 143-145
- Risk: Script assumes $HOME is always available and writable
- Priority: Low - less likely in typical usage

## Dependencies at Risk

**Zellij version pinning strategy:**
- Risk: Latest release downloads not pinned; breaking changes possible
- Impact: New container builds could fail if Zellij introduces incompatibilities
- Migration plan: Pin to specific version tag in Containerfile line 14, e.g., `v0.40.0` instead of `latest`

**npm package get-shit-done-cc:**
- Risk: `@latest` tag without verification; package could be compromised or have breaking changes
- Impact: Container startup fails silently if package fails to install
- Migration plan: Pin to known-good version, monitor for updates, implement version checking

**curl-based installer scripts:**
- Risk: starship.rs/install.sh, astral.sh/uv/install.sh, claude.ai/install.sh are external URLs
- Impact: Service outages or domain takeovers would break container builds
- Migration plan: Cache installer scripts in repo, use checksums, consider package manager alternatives

## Missing Critical Features

**No health check mechanism:**
- Problem: No way to verify container/services are actually ready
- Blocks: Reliable session attachment, debugging of startup issues
- Recommendation: Implement health endpoint or status check command

**No session cleanup mechanism:**
- Problem: Long-lived sessions accumulate; no garbage collection for stopped containers
- Blocks: Managing many sessions becomes difficult
- Recommendation: Add `--gc` flag or automatic cleanup for old containers

**No logging or audit trail:**
- Problem: Container creation/attachment events not logged
- Blocks: Debugging issues, understanding session lifecycle
- Recommendation: Add logging to key operations, optional verbose mode

**No configuration file support:**
- Problem: Mount paths and defaults only via CLI args and env vars
- Blocks: Complex multi-project setups, persistent preferences
- Recommendation: Support `~/.agent-session/config` with TOML/YAML format

## Deployment & Reliability Concerns

**No version pinning in Containerfile:**
- Problem: Uses latest versions of Debian packages, tools without checksums
- Impact: Builds are non-reproducible; same Containerfile produces different results over time
- Fix approach: Pin specific package versions, verify checksums for downloaded tools

**Container image size not optimized:**
- Problem: Full Debian bookworm-slim used; no layer consolidation
- Impact: Initial pull and disk usage are larger than necessary
- Fix approach: Consolidate RUN commands, remove unnecessary tools, use distroless or Alpine base

**No container resource limits:**
- Problem: Sessions can consume unlimited CPU, memory, disk
- Impact: Runaway process in one session could impact host
- Fix approach: Add --memory, --cpus, and --storage-driver resource limits to podman run

