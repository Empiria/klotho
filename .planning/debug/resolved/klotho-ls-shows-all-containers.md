---
status: resolved
trigger: "klotho ls shows all podman containers instead of only klotho-related containers"
created: 2026-01-28T00:00:00Z
updated: 2026-01-28T00:00:00Z
---

## Current Focus

hypothesis: The filter condition in list_containers is too broad - `name.contains("-")` matches any container with a hyphen
test: Read the filter code in container.rs line 216
expecting: Overly broad condition confirmed
next_action: Fix filter to only match klotho-prefixed containers and add label for reliable filtering

## Symptoms

expected: klotho ls should only show containers that are related to klotho
actual: klotho ls shows ALL podman containers on the system
errors: none - functional but shows too much
reproduction: run `klotho ls` when other non-klotho podman containers exist
started: unknown

## Eliminated

## Evidence

- timestamp: 2026-01-28T00:00:00Z
  checked: container.rs line 216 filter condition
  found: `if name.starts_with("klotho-session-") || name.contains("-")` - the `name.contains("-")` matches virtually all containers
  implication: This is the root cause - any container with a hyphen in its name is included

- timestamp: 2026-01-28T00:00:00Z
  checked: start.rs container creation
  found: New containers always use `klotho-session-{agent}-{name}` naming, no labels applied
  implication: Can fix by restricting filter to `klotho-` prefix; could also add labels for future robustness

## Resolution

root_cause: In container.rs:216, the filter `name.contains("-")` is far too broad - it matches any container whose name contains a hyphen, which is nearly all containers. This was intended to catch "legacy" containers but has no way to distinguish them from unrelated containers.
fix: 1) Add `--label klotho=true` when creating containers. 2) Filter by label in list_containers. 3) Fall back to `klotho-` prefix matching for containers created before labeling.
verification: cargo build succeeds; filter now uses label-based filtering + klotho- prefix fallback instead of overly broad hyphen check
files_changed: [src/container.rs, src/commands/start.rs]
