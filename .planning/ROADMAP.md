# Roadmap: Agent Session

## Overview

Transform a personal tool into a team-ready containerized agent environment. The journey starts with portability and security audits to remove personal assumptions, builds a multi-agent foundation that separates orchestration from agent definitions, adds essential session management features, and finishes with comprehensive documentation that enables colleague adoption in under 5 minutes.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Release Audit** - Remove personal assumptions and security risks
- [x] **Phase 2: Agent Abstraction** - Extract agent definitions into config-driven architecture
- [x] **Phase 3: Multi-Agent Support** - Add OpenCode and interactive agent selection
- [x] **Phase 4: Session Management** - Essential container lifecycle commands
- [x] **Phase 5: Documentation** - Quick start guide and usage reference
- [x] **Phase 6: Rename to Klotho** - Rebrand project and CLI from agent-session to klotho
- [ ] **Phase 7: Rust Rewrite** - Migrate bash script to Rust for better maintainability and single-binary distribution

## Phase Details

### Phase 1: Release Audit
**Goal**: Codebase is portable and secure for team distribution
**Depends on**: Nothing (first phase)
**Requirements**: REL-01, REL-02, REL-03
**Success Criteria** (what must be TRUE):
  1. Codebase contains zero hardcoded usernames or machine-specific absolute paths
  2. No API keys, secrets, or credentials exist in repository
  3. Tool runs successfully on fresh Debian container with only documented prerequisites
  4. All environment assumptions are documented with verification commands
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Security and portability audit (Gitleaks, ShellCheck)
- [x] 01-02-PLAN.md — Fresh environment verification and PREREQUISITES.md
- [x] 01-03-PLAN.md — Gap closure: conditional mounting of optional directories

### Phase 2: Agent Abstraction
**Goal**: Agent definitions are config-driven, enabling easy addition of new agents
**Depends on**: Phase 1
**Requirements**: AGT-05, AGT-01
**Success Criteria** (what must be TRUE):
  1. Agent definition format exists with template and documentation
  2. Claude agent is defined via agents/claude.conf with installation, paths, and commands
  3. Containerfile uses multi-stage builds with base stage and agent-specific stages
  4. Adding a new agent requires only config file and Containerfile stage, not orchestration logic changes
  5. User can run Claude sessions using the abstracted architecture (no regression)
**Plans**: 4 plans

Plans:
- [x] 02-01-PLAN.md — Create agent config format and Claude config file
- [x] 02-02-PLAN.md — Multi-stage Containerfile and config-driven build script
- [x] 02-03-PLAN.md — Update orchestration and verify end-to-end functionality
- [x] 02-04-PLAN.md — Gap closure: dynamic wrapper names and config cleanup

### Phase 3: Multi-Agent Support
**Goal**: Users can select and run multiple agent types interactively or via flags
**Depends on**: Phase 2
**Requirements**: AGT-02, AGT-03, AGT-04
**Success Criteria** (what must be TRUE):
  1. OpenCode agent runs in sessions alongside Claude (separate containers)
  2. User can select agent via interactive menu when starting session without flags
  3. User can specify agent via --agent flag for scripting and automation
  4. Interactive menu shows agent descriptions and defaults intelligently
  5. Both agents install their dependencies correctly and launch without conflicts
**Plans**: 3 plans

Plans:
- [x] 03-01-PLAN.md — OpenCode agent definition (config, MCP config, Containerfile stage)
- [x] 03-02-PLAN.md — Interactive agent selection with build status detection
- [x] 03-03-PLAN.md — OpenCode runtime integration and end-to-end verification

### Phase 4: Session Management
**Goal**: Users can manage container lifecycle without raw podman commands
**Depends on**: Phase 3
**Requirements**: SES-01, SES-02, SES-03, SES-04
**Success Criteria** (what must be TRUE):
  1. User can stop a running agent session and container stops cleanly
  2. User can restart a stopped agent session and reattach to existing Zellij session
  3. User can list all agent sessions showing name, agent type, and status (running/stopped)
  4. User can remove stopped containers to reclaim disk space
  5. Commands provide clear feedback and handle edge cases (no such container, already stopped)
**Plans**: 2 plans

Plans:
- [x] 04-01-PLAN.md — Refactor to subcommand structure with stop and restart commands
- [x] 04-02-PLAN.md — Implement ls and rm commands for session listing and cleanup

### Phase 5: Documentation
**Goal**: Colleague can install and successfully run first command in under 5 minutes
**Depends on**: Phase 4
**Requirements**: DOC-01, DOC-02, DOC-03
**Success Criteria** (what must be TRUE):
  1. Quick start guide exists with clear steps from zero to first successful session
  2. Installation guide lists all prerequisites with verification commands (podman --version, etc.)
  3. Usage reference documents all commands, flags, and examples with expected output
  4. Documentation tested with fresh-eye colleague on clean machine
  5. Common errors have troubleshooting entries with solutions
**Plans**: 2 plans

Plans:
- [x] 05-01-PLAN.md — README foundation: overview, prerequisites, concepts, quick start
- [x] 05-02-PLAN.md — Command reference and troubleshooting sections

### Phase 6: Rename to Klotho
**Goal**: Project and CLI tool renamed from "agent-session" to "klotho"
**Depends on**: Phase 5
**Requirements**: None (new scope)
**Success Criteria** (what must be TRUE):
  1. CLI command is `klotho` (not `agent-session`)
  2. Container images use `klotho-<agent>:latest` naming pattern
  3. All documentation references the new name
  4. Repository can be renamed without breaking functionality
  5. Existing sessions continue to work during transition
**Plans**: 3 plans

Plans:
- [x] 06-01-PLAN.md — Core CLI rename (script file, help text, XDG paths, symlink)
- [x] 06-02-PLAN.md — Build system and container naming (image tags, dual detection)
- [x] 06-03-PLAN.md — Documentation updates (README with name explanation, PREREQUISITES, AGENTS.md)

### Phase 7: Rust Rewrite
**Goal**: CLI tool rewritten in Rust for better maintainability and single-binary distribution
**Depends on**: Phase 6
**Requirements**: None (new scope)
**Success Criteria** (what must be TRUE):
  1. Rust CLI provides all commands from bash version (start, stop, restart, ls, rm) plus new build/rebuild commands
  2. Single static binary with no runtime dependencies (musl on Linux)
  3. Argument parsing and help text matches or improves on bash version
  4. Config file loading works identically to bash version
  5. Podman container management works correctly, with Docker fallback via --runtime flag
  6. Zellij session attachment works correctly
  7. Legacy naming migration continues to work during transition
  8. GitHub releases provide pre-built binaries for Linux, macOS, and Windows
  9. curl | sh installer script available for quick installation
**Plans**: 8 plans

Plans:
- [ ] 07-01-PLAN.md — Rust project setup and CLI structure with clap
- [ ] 07-02-PLAN.md — Config loading and container runtime abstraction
- [ ] 07-03-PLAN.md — Embedded resources (Containerfile, configs)
- [ ] 07-04-PLAN.md — Start command implementation
- [ ] 07-05-PLAN.md — Stop, restart, ls, rm commands
- [ ] 07-06-PLAN.md — Build and rebuild commands with spinner progress
- [ ] 07-07-PLAN.md — GitHub Actions CI/CD and installer script
- [ ] 07-08-PLAN.md — End-to-end verification checkpoint

**Details:**
Rewrite the klotho bash script (~855 lines) in Rust to improve maintainability and enable single-binary distribution. The bash script has grown complex with argument parsing, config validation, container management, and legacy migration logic. Rust provides better error handling, proper data structures, and distributes as a single binary via GitHub releases or cargo install. **Expanded scope:** Add build and rebuild commands (not in bash version), Docker fallback support, and curl | sh installer.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Release Audit | 3/3 | Complete | 2026-01-26 |
| 2. Agent Abstraction | 4/4 | Complete | 2026-01-26 |
| 3. Multi-Agent Support | 3/3 | Complete | 2026-01-26 |
| 4. Session Management | 2/2 | Complete | 2026-01-27 |
| 5. Documentation | 2/2 | Complete | 2026-01-27 |
| 6. Rename to Klotho | 3/3 | Complete | 2026-01-27 |
| 7. Rust Rewrite | 0/8 | Not started | - |
