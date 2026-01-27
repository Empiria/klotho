# Coding Conventions

**Analysis Date:** 2026-01-26

## Naming Patterns

**Files:**
- Shell scripts follow kebab-case: `agent-session`, `entrypoint.sh`
- Executable scripts are named without language extension, or with `.sh` extension for sourced scripts
- Configuration files use descriptive names: `Containerfile` (Podman/Docker build specification)

**Variables:**
- Environment variables are UPPERCASE_WITH_UNDERSCORES: `SESSION_NAME`, `CONTAINER_NAME`, `MOUNTS`, `WORKDIR`, `AGENT_SESSION_MOUNTS`
- Local variables use UPPERCASE_WITH_UNDERSCORES within functions: `FIRST_DIR`, `PATHS`, `EXTRA_MOUNTS`
- Arrays use UPPERCASE naming: `PATHS=()`, `MOUNT_PATHS`
- Loop variables use lowercase with underscores: `path`, `item`, `mount_path`, `abs_path`, `dir_name`

**Functions:**
- Function names use lowercase_with_underscores: `show_help()`, `attach_zellij()`
- Functions are clearly defined with `functionname() {}` syntax

**Constants:**
- Hardcoded strings that represent constants use UPPERCASE: used in comparisons like `^${CONTAINER_NAME}$`

## Code Style

**Formatting:**
- Uses standard bash formatting conventions
- No tabs in indentation; uses 4-space indentation for nested blocks
- Line length typically kept under 100 characters, with continuation on next line when necessary
- Consistent spacing around operators and control structures

**Linting:**
- No automated linting/formatting tools detected
- Project follows bash best practices: `set -euo pipefail` at script start for error handling
- Uses `shellcheck` compatible patterns (though no config file present)

**Shell Safety:**
- Scripts start with proper shebangs: `#!/bin/bash` or `#!/usr/bin/env fish`
- Error handling: Uses `set -e` (exit on error) and `set -euo pipefail` (error on undefined vars, pipe failures)
- Proper quoting of variables: `"$CONTAINER_NAME"`, `"${PATHS[@]}"` to prevent word splitting
- Array handling: Uses proper array expansion `"${PATHS[@]}"` instead of `$PATHS`

## Import Organization

**Not Applicable:** Bash scripts do not use imports/modules in the traditional sense. External tools are invoked via commands.

**External Commands Used:**
- `podman` (container orchestration)
- `zellij` (terminal multiplexer)
- `realpath` (path resolution)
- `basename` (filename extraction)
- `grep`, `sed` (text processing)
- `mkdir`, `ln` (filesystem operations)

## Error Handling

**Patterns:**
- Early exit on errors: `set -e` and `set -euo pipefail` at script start
- Explicit exit codes: `exit 0` for success, `exit 1` for errors
- Error messages sent to stderr: `echo "error: ..." >&2`
- Warning messages to stderr: `echo "warning: ..." >&2`
- Wrapped conditionals check for command existence before execution
- Error validation for required arguments with fallback messaging

**Example from codebase (`agent-session` lines 50-55):**
```bash
if [[ -z "${2:-}" || "$2" == -* ]]; then
    echo "error: -n/--name requires a session name" >&2
    echo >&2
    show_help >&2
    exit 1
fi
```

## Logging

**Framework:** Standard bash `echo` for logging

**Patterns:**
- Info messages to stdout: `echo "Starting new session '$SESSION_NAME'..."`
- Error/warning messages to stderr: `echo "error: ..." >&2`
- Status messages use clear, human-readable text
- Container logs inspectable via `podman logs agent-NAME`
- Within entrypoint, status messages confirm readiness: `echo "Claude Code $(claude --version) ready"`

**Example from codebase (`entrypoint.sh` lines 31-32):**
```bash
echo "Installing get-shit-done..."
echo "Claude Code $(claude --version) ready"
```

## Comments

**When to Comment:**
- Comments explain "why" not "what" when intent is not obvious from code
- Comments document non-standard behavior or workarounds
- Comments explain temporary decisions or known limitations
- Complex shell logic (like ANSI color stripping) is commented

**Example from codebase (`agent-session` lines 76-77):**
```bash
# Strip ANSI color codes when checking for existing sessions
if podman exec "$CONTAINER_NAME" zellij list-sessions 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep -q "^$SESSION_NAME "; then
```

**Example from codebase (`entrypoint.sh` lines 16-17):**
```bash
if [[ -L "$item" && ! -e "$item" ]]; then
    # Broken symlink - create directory instead
    mkdir -p ~/.claude/"$name"
```

## Control Flow

**Conditionals:**
- Bash conditionals use `[[ ]]` (preferred) over `[ ]` for better handling of unset variables
- Single bracket test `[ ]` used sparingly in non-bash contexts
- Negation uses `!` prefix clearly: `[[ ! -e "$item" ]]`
- Early returns and exits reduce nesting: functions check preconditions first

**Case Statements:**
- Used for option parsing in `agent-session` lines 43-70
- Each case branch is clearly labeled and properly escaped for pipe characters
- Pattern matching accounts for both long and short options: `-h|--help`

**Example from codebase (`agent-session` lines 43-70):**
```bash
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -n|--name)
            if [[ -z "${2:-}" || "$2" == -* ]]; then
                echo "error: -n/--name requires a session name" >&2
                exit 1
            fi
            SESSION_NAME="$2"
            shift 2
            ;;
        *)
            PATHS+=("$1")
            shift
            ;;
    esac
done
```

## Function Design

**Size:** Functions kept to single logical responsibility. `show_help()` only displays help; `attach_zellij()` handles session attachment.

**Parameters:**
- Functions receive parameters via global variables set before invocation (bash convention)
- No parameter passing in function definitions
- Functions use positional parameters where needed via bash expansion

**Return Values:**
- Functions primarily return exit codes (0 for success)
- Complex return values communicated through global variables
- Errors cause immediate script exit via `set -e` rather than checking return codes

## Module Organization

**Barrel Files:** Not applicable to bash scripts

**Exports:**
- No module system; scripts are self-contained
- Sourced scripts (in entrypoint) make their effects visible to caller through environment modifications
- Child processes inherit environment: `exec "$@"` passes through all prior setup

## Array Handling

**Pattern:**
- Arrays declared explicitly: `PATHS=()`
- Elements appended with `+=`: `PATHS+=("$1")`
- Iteration uses proper expansion: `for path in "${PATHS[@]}"`
- Loop counter used when needed: `while [[ $# -gt 0 ]]`

**Example from codebase (`agent-session` lines 117-122):**
```bash
MOUNTS=""
for path in "${PATHS[@]}"; do
    abs_path=$(realpath "$path")
    dir_name=$(basename "$abs_path")
    MOUNTS="$MOUNTS -v $abs_path:/workspace/$dir_name:Z"
    [[ -z "$FIRST_DIR" ]] && FIRST_DIR="/workspace/$dir_name"
done
```

## Path Handling

**Pattern:**
- Absolute paths preferred: `realpath "$path"` converts relative to absolute
- Path variables quoted to handle spaces: `"$path"`, `"${PATHS[@]}"`
- Use variable expansion for path manipulation: `${variable%suffix}` or `${variable#prefix}` patterns
- Container paths use consistent prefixes: `/workspace/`, `/config/`, `/home/agent/`

---

*Convention analysis: 2026-01-26*
