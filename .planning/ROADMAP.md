# Roadmap: Agent Session

## Overview

Transform a personal tool into a team-ready containerized agent environment. The journey starts with portability and security audits to remove personal assumptions, builds a multi-agent foundation that separates orchestration from agent definitions, adds essential session management features, and finishes with comprehensive documentation that enables colleague adoption in under 5 minutes.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Release Audit** - Remove personal assumptions and security risks
- [ ] **Phase 2: Agent Abstraction** - Extract agent definitions into config-driven architecture
- [ ] **Phase 3: Multi-Agent Support** - Add OpenCode and interactive agent selection
- [ ] **Phase 4: Session Management** - Essential container lifecycle commands
- [ ] **Phase 5: Documentation** - Quick start guide and usage reference

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
**Plans**: 2 plans

Plans:
- [ ] 01-01-PLAN.md — Security and portability audit (Gitleaks, ShellCheck)
- [ ] 01-02-PLAN.md — Fresh environment verification and PREREQUISITES.md

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
**Plans**: TBD

Plans:
- [ ] TBD

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
**Plans**: TBD

Plans:
- [ ] TBD

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
**Plans**: TBD

Plans:
- [ ] TBD

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
**Plans**: TBD

Plans:
- [ ] TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Release Audit | 0/2 | Planned | - |
| 2. Agent Abstraction | 0/TBD | Not started | - |
| 3. Multi-Agent Support | 0/TBD | Not started | - |
| 4. Session Management | 0/TBD | Not started | - |
| 5. Documentation | 0/TBD | Not started | - |
