# Testing Patterns

**Analysis Date:** 2026-01-26

## Test Framework

**Status:** No automated testing framework detected

**Current Approach:**
- No test runner configured (Jest, Vitest, pytest, BATS, Shellcheck, etc.)
- Manual testing via container invocation
- Validation through script execution in containerized environment

**Why:** The codebase consists of shell scripts for container orchestration and initialization. Testing is primarily through:
1. Container lifecycle verification
2. Manual session attachment/detachment
3. Integration testing against podman/Zellij runtime

## Test Organization

**Location:**
- No test directory structure exists (no `/tests`, `/test`, `/__tests__` directories)
- No test files detected (no `*.test.sh`, `*.spec.sh`, `*_test.sh` patterns)

**Current Coverage:**
- `agent-session` (`/home/owen/projects/personal/agent-session/agent-session`): 154 lines - untested
- `entrypoint.sh` (`/home/owen/projects/personal/agent-session/entrypoint.sh`): 39 lines - untested

## Manual Testing Approach

The project uses manual verification patterns documented in help text and container logs:

**From `agent-session` help (lines 5-36):**
```bash
# List running sessions
podman ps --filter "name=agent-"

# Stop a session
podman stop agent-NAME && podman rm agent-NAME

# Inside the container
claude --dangerously-skip-permissions
```

**Key Test Scenarios (inferred from code):**

1. **Session Creation:**
   - Verify container starts with `podman run`
   - Check working directory is set correctly
   - Confirm mounts are accessible

2. **Session Reattachment:**
   - Existing running container → attach to existing Zellij session
   - Stopped container → start and attach
   - Verify `zellij list-sessions` filtering works correctly

3. **Argument Parsing:**
   - Single path: `agent-session ~/projects/repo`
   - Multiple paths: `agent-session ~/api ~/libs`
   - Named session: `agent-session -n frontend`
   - Named session with paths: `agent-session -n backend ~/api`
   - Invalid option handling: `-n` without value
   - Help flag: `-h`, `--help`

4. **Mount Handling:**
   - Path validation (file/dir exists before mounting)
   - Extra mounts from `AGENT_SESSION_MOUNTS` env var
   - Volume permissions: read-only (`:ro`) for config, read-write (`:Z`) for workspace
   - Absolute path resolution: `realpath` converts relative paths

5. **Container Initialization:**
   - `entrypoint.sh` sets up `~/.claude` symlinks correctly
   - Handles broken symlinks by creating directories
   - Copies Zellij config to home directory
   - Installs get-shit-done plugin if missing
   - Reports Claude version on ready

## Error Scenarios (Unverified)

The code handles but does not have tests for:
- Non-existent paths: `agent-session ~/nonexistent`
- Permission denied on mounts
- Container name collisions
- Zellij session conflicts
- Network issues during plugin installation
- Missing `podman` command
- Missing `zellij` command

## Testing Gaps

**High Priority:**
1. **Path validation** - Lines 107-112 in `agent-session` validate paths exist; no test coverage
   ```bash
   for path in "${PATHS[@]}"; do
       if [[ ! -e "$path" ]]; then
           echo "error: path does not exist: $path" >&2
           exit 1
       fi
   done
   ```

2. **Mount construction** - Lines 117-122 build complex volume flags; no verification tests
   ```bash
   MOUNTS="$MOUNTS -v $abs_path:/workspace/$dir_name:Z"
   ```

3. **ANSI stripping** - Line 77 strips color codes from session list; untested edge case
   ```bash
   zellij list-sessions 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -q "^$SESSION_NAME "
   ```

4. **Option parsing** - Lines 43-70 parse arguments with complex error handling; no test coverage

5. **Container lifecycle** - Lines 85-98 implement three-way state machine (running/stopped/new); untested

6. **Entrypoint setup** - Lines 9-23 in `entrypoint.sh` handle symlink/directory creation; no tests

## Recommended Testing Additions

**Shell Script Testing Framework:**
Would benefit from BATS (Bash Automated Testing System) or similar:

```bash
# Example BATS test structure (not currently present)
@test "agent-session creates container with correct mounts" {
    run agent-session -n test-session /tmp/test
    [ "$status" -eq 0 ]
    podman ps --format "{{.Names}}" | grep -q "^agent-test-session$"
}

@test "agent-session rejects non-existent paths" {
    run agent-session /nonexistent/path
    [ "$status" -eq 1 ]
    [[ "$output" == *"path does not exist"* ]]
}

@test "entrypoint.sh creates ~/.claude directory structure" {
    # Run in container
    podman exec agent-test test -d ~/.claude
    [ $? -eq 0 ]
}
```

**Integration Testing:**
- Verify container orchestration with real `podman`
- Test Zellij session attachment/detachment
- Verify environment variable passing
- Test multi-path mounting scenarios

**Validation Scripts:**
None currently exist. Could create:
- `scripts/validate-mounts.sh` - Verify all mounts are accessible
- `scripts/test-argument-parsing.sh` - Test various option combinations
- `scripts/test-container-lifecycle.sh` - Test create/start/stop/reattach flows

## Test Execution (Current State)

**No automated test command exists.**

**Manual testing via:**
```bash
# Build container
podman build -f Containerfile -t claude-agent

# Test session creation
./agent-session -n test-session ~

# Verify in another terminal
podman ps --filter "name=agent-test-session"
podman logs agent-test-session

# Clean up
podman stop agent-test-session && podman rm agent-test-session
```

---

*Testing analysis: 2026-01-26*
