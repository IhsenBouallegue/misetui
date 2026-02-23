# External Integrations

**Analysis Date:** 2026-02-23

## APIs & External Services

**mise CLI Tool:**
- `mise` (https://mise.jdx.dev) - Version manager and dev tool orchestration
  - Subprocess calls via `tokio::process::Command`
  - Integration point: `src/mise.rs`
  - Auth: None (local CLI tool, no authentication)

## Data Storage

**Databases:**
- Not used - No persistent database

**File Storage:**
- Local filesystem only - mise configuration files and tool installations live on user's filesystem
- Read via `mise config ls -J` command
- No direct file I/O in misetui; all file management delegated to mise CLI

**Caching:**
- None - All data is fetched fresh from mise on demand
- Highlight indices cached in memory during filtering (`tools_hl: Vec<Vec<usize>>` pattern in `src/app.rs`)
- User state (selected items, search filters) held in `App` struct only during session

## Authentication & Identity

**Auth Provider:**
- None - misetui is a terminal interface layer with no authentication
- Inherits all authentication/authorization from `mise` tool

## Monitoring & Observability

**Error Tracking:**
- None - color-eyre provides local error reporting to stderr only

**Logs:**
- Console-only: Errors printed to stderr via `color-eyre`
- Status messages shown as ephemeral toasts on TUI
- No log files or external logging services

## CI/CD & Deployment

**Hosting:**
- Not applicable - CLI tool distributed via Cargo

**CI Pipeline:**
- Not detected - Repository has no CI configuration files

**Distribution:**
- Cargo crates.io - Published as `misetui` crate
- Manual installation from GitHub source or via `cargo install misetui`

## Environment Configuration

**Required env vars:**
- None required at runtime
- System `PATH` must contain `mise` executable

**Secrets location:**
- Not applicable - No secrets or credentials used

## Subprocess Calls & External Processes

**mise Commands Executed:**
- `mise ls -J` - List installed tools (JSON output)
- `mise registry -J` - Fetch plugin registry (JSON output)
- `mise config ls -J` - List configuration files (JSON output)
- `mise doctor` - Run diagnostics (plain text output)
- `mise ls-remote [tool]` - List available versions for tool
- `mise install [tool]@[version]` - Install tool version
- `mise uninstall [tool]@[version]` - Remove tool version
- `mise upgrade [tool]` - Update tool to latest version
- `mise outdated -J` - List outdated tools (JSON output)
- `mise tasks ls -J` - List available tasks (JSON output)
- `mise env --json-extended` - Fetch environment variables (JSON output)
- `mise settings ls -J` - List settings (JSON output)
- `mise use --global [tool]@[version]` - Set global tool version
- `mise prune` - Remove unused tool versions

**Implementation:** `src/mise.rs` contains async wrapper functions using `tokio::process::Command`

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Network Connectivity

**Network Requirements:**
- None required for misetui itself
- mise CLI may require network access for downloading tools or checking registries (handled by mise, not misetui)

---

*Integration audit: 2026-02-23*
