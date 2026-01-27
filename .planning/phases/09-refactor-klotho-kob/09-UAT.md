---
status: testing
phase: 09-refactor-klotho-kob
source: [09-01-SUMMARY.md, 09-02-SUMMARY.md]
started: 2026-01-27T17:20:00Z
updated: 2026-01-27T17:20:00Z
---

## Current Test

number: 6
name: Legacy env vars removed
expected: |
  Running `grep -r "KLOTHO_KOB\|AGENT_SESSION" src/` returns no matches.
awaiting: user response

## Tests

### 1. --linked-dir flag repeatable
expected: Running `cargo run -- start --help` shows `--linked-dir` option. Multiple `--linked-dir` flags accepted without parse error.
result: pass

### 2. Help text describes symlink purpose
expected: Running `cargo run -- start --help` shows `--linked-dir` with description mentioning symlink resolution.
result: pass

### 3. Bash script deleted
expected: Running `ls klotho` in repo root fails (file not found). Only Rust binary exists.
result: pass

### 4. KLOTHO_LINKED_DIRS environment variable works
expected: Running `KLOTHO_LINKED_DIRS=/tmp cargo run -- start test-session .` attempts container start with mount for /tmp (may fail at container level but CLI parsing works).
result: pass

### 5. CLI flags merge with env var
expected: Running `KLOTHO_LINKED_DIRS=/tmp cargo run -- start --linked-dir /var/tmp test-session .` combines both directories.
result: pass

### 6. Legacy env vars removed
expected: Running `grep -r "KLOTHO_KOB\|AGENT_SESSION" src/` returns no matches.
result: [pending]

### 7. README documents KLOTHO_LINKED_DIRS
expected: README.md contains KLOTHO_LINKED_DIRS documentation with usage examples showing both env var and --linked-dir flag.
result: [pending]

## Summary

total: 7
passed: 5
issues: 0
pending: 2
skipped: 0

## Gaps

[none yet]
