# Requirements

**Project:** agent-session
**Version:** v1
**Last updated:** 2026-01-26

## v1 Requirements

### Release Readiness

- [ ] **REL-01**: Codebase contains no hardcoded usernames or absolute paths specific to one machine
- [ ] **REL-02**: No API keys, secrets, or credentials committed to repository
- [ ] **REL-03**: Tool runs successfully on fresh container with only documented prerequisites

### Session Management

- [x] **SES-01**: User can stop a running agent session
- [x] **SES-02**: User can restart a stopped agent session
- [x] **SES-03**: User can list all agent sessions with their status (running/stopped)
- [x] **SES-04**: User can remove old/stopped containers to clean up

### Agent Support

- [ ] **AGT-01**: User can run sessions with Claude Code agent
- [x] **AGT-02**: User can run sessions with opencode agent
- [x] **AGT-03**: User can select agent via interactive menu when starting session
- [x] **AGT-04**: User can specify agent via `--agent` flag for scripting/automation
- [ ] **AGT-05**: Adding a new agent requires only config/Containerfile changes, not orchestration logic

### Documentation

- [x] **DOC-01**: Quick start guide gets colleague to first successful command in <5 minutes
- [x] **DOC-02**: Installation guide lists all prerequisites with verification commands
- [x] **DOC-03**: Usage reference documents all commands and flags

---

## v2 Requirements (Deferred)

### Session Management
- Auto-restart on crash
- Bulk operations (stop all, clean all)
- Logs access command

### Documentation
- Troubleshooting guide

### Developer Experience
- Shell completion (bash/zsh/fish)
- ShellCheck linting integration
- BATS automated tests
- CI/CD pipeline

---

## Out of Scope

- **Full TUI (lazydocker-style)** - overengineering for small team, interactive menu is enough
- **Database for state** - containers already track state, query podman directly
- **User authentication** - small team trust boundary, unnecessary complexity
- **Plugin system** - premature, add agents by editing config instead
- **GUI/Web UI** - scope creep, users are CLI-comfortable
- **Remote container management** - SSH works, focus on local
- **Complex dependency graphs** - keep agents independent

---

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| REL-01 | Phase 1 | Complete |
| REL-02 | Phase 1 | Complete |
| REL-03 | Phase 1 | Complete |
| AGT-05 | Phase 2 | Complete |
| AGT-01 | Phase 2 | Complete |
| AGT-02 | Phase 3 | Complete |
| AGT-03 | Phase 3 | Complete |
| AGT-04 | Phase 3 | Complete |
| SES-01 | Phase 4 | Complete |
| SES-02 | Phase 4 | Complete |
| SES-03 | Phase 4 | Complete |
| SES-04 | Phase 4 | Complete |
| DOC-01 | Phase 5 | Complete |
| DOC-02 | Phase 5 | Complete |
| DOC-03 | Phase 5 | Complete |

---

*Requirements defined: 2026-01-26*
