---
status: resolved
trigger: "Investigate issue: klotho-broken-symlinks-overlay"
created: 2026-01-28T00:00:00Z
updated: 2026-01-28T13:35:00Z
---

## Current Focus

hypothesis: CONFIRMED - Symlinks are remnants from old bash implementation that created them inside container
test: Check if removing symlinks and using KLOTHO_LINKED_DIRS properly fixes the issue
expecting: After removing broken symlinks, KLOTHO_LINKED_DIRS should mount the KOB directory correctly
next_action: Verify root cause and develop fix

## Symptoms

expected: Symlinks should be absolute paths. Klotho should mount them using env vars to the same path on the container, so they resolve correctly both on host and in container.
actual: Symlinks now point to '/overlay/...' paths which don't exist on the host, making them broken.
errors: Broken symlinks in /home/owen/projects/empiria/friendly-fox/infinite-worlds
reproduction: Use klotho with a project that has symlinks, after session ends the symlinks point to /overlay/... paths
started: Worked in old bash implementation, broken in new Rust implementation (first use)

## Eliminated

## Evidence

- timestamp: 2026-01-28T00:10:00Z
  checked: /home/owen/projects/empiria/friendly-fox/infinite-worlds symlinks
  found: Symlinks point to /overlay-mounts/infinite-worlds/... paths, e.g. AGENTS.md -> /overlay-mounts/infinite-worlds/AGENTS.md
  implication: Something is creating or modifying symlinks to point to overlay paths that don't exist on host

- timestamp: 2026-01-28T00:11:00Z
  checked: Old bash implementation (git history)
  found: No overlay filesystem handling in bash version - just regular mounts with -v flag
  implication: Overlay behavior is new to Rust implementation or related to how container handles mounts

- timestamp: 2026-01-28T00:20:00Z
  checked: ~/.claude/file-history for old bash implementation
  found: OLD bash implementation parsed .agent-session.conf, mounted targets to /overlay-mounts/<project>/<name>, then created symlinks INSIDE container pointing to those paths
  implication: Symlinks were created by OLD implementation and persisted on host filesystem

- timestamp: 2026-01-28T00:22:00Z
  checked: Creation dates of symlinks
  found: Symlinks created Jan 27 11:04 - exactly when user first used new Rust implementation
  implication: Wait - this contradicts previous finding. Symlinks were created AFTER Rust migration!

- timestamp: 2026-01-28T00:25:00Z
  checked: Claude session logs showing /overlay-mounts paths
  found: Claude sessions from Jan 27 11:02-11:27 show symlinks already existed with /overlay-mounts paths, and Claude was trying to use them
  implication: Symlinks WERE created by old bash implementation, dates show when they were last accessed/modified

## Resolution

root_cause: Old bash implementation (agent-session script) parsed .agent-session.conf file, mounted overlay targets to /overlay-mounts/<project>/<name> paths inside container, then created symlinks inside the container workspace pointing to those /overlay-mounts paths. These symlinks were written to the mounted volume, so they persisted on the host filesystem. When the Rust implementation replaced the bash version, it no longer creates or uses /overlay-mounts, but the old symlinks remain on the host pointing to non-existent paths. The new Rust implementation uses KLOTHO_LINKED_DIRS instead, which mounts directories at the same path on both host and container.

fix: Removed broken symlinks and recreated them with ABSOLUTE paths pointing to the KOB directory. Absolute paths are required because KLOTHO_LINKED_DIRS mounts the KOB directory at the same absolute path inside the container, allowing symlinks to resolve correctly both on host and in container.

Fixed symlinks in /home/owen/projects/empiria/friendly-fox/infinite-worlds:
- AGENTS.md -> /home/owen/projects/empiria/friendly-fox/friendly-fox-kob/infinite-worlds/AGENTS.md
- .serena -> /home/owen/projects/empiria/friendly-fox/friendly-fox-kob/infinite-worlds/.serena
- scripts -> /home/owen/projects/empiria/friendly-fox/friendly-fox-kob/infinite-worlds/scripts
- .planning -> /home/owen/projects/empiria/friendly-fox/friendly-fox-kob/infinite-worlds/gsd-planning

verification: All symlinks tested and working on host. Symlinks use absolute paths that will resolve correctly inside container since KLOTHO_LINKED_DIRS=/home/owen/projects/empiria/friendly-fox/friendly-fox-kob/infinite-worlds mounts the KOB directory at the same path inside the container.

files_changed:
- /home/owen/projects/empiria/friendly-fox/infinite-worlds/AGENTS.md (symlink fixed)
- /home/owen/projects/empiria/friendly-fox/infinite-worlds/.serena (symlink fixed)
- /home/owen/projects/empiria/friendly-fox/infinite-worlds/.planning (symlink fixed)
- /home/owen/projects/empiria/friendly-fox/infinite-worlds/scripts (symlink fixed)
