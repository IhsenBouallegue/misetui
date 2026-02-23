# Phase 4: Inline Editor - Context

**Gathered:** 2026-02-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can add, edit, and delete tools, env vars, and tasks in any `.mise.toml` directly from the TUI without touching a text editor. File writes are atomic via `toml_edit` with round-trip preservation of formatting and comments. After a successful write, the app refreshes config and tools state automatically.

</domain>

<decisions>
## Implementation Decisions

### Editor layout & navigation
- Centered popup (consistent with existing wizard/help popup patterns), main view dimmed behind
- Three sub-tabs at the top: Tools / Env / Tasks — switch between them with h/l or Tab
- Only one section visible at a time
- Each section displays rows as a clean table with columns (e.g., Name | Version | Status for tools)
- Editor always starts on the Tools tab regardless of context

### Input method for values
- Inline editing: cursor appears directly in the table cell, type to edit in-place
- Enter to confirm edit, Esc to cancel
- When adding a new tool, tool name selection uses the existing registry fuzzy search (same as current Tools tab behavior)
- Env var key and value entry method is at Claude's discretion (two-step or single-line)
- Task commands are single-line only — complex scripts belong in files

### Version selection flow
- Reuse the existing version picker popup from the Tools tab
- When editing a tool's version, picker opens pre-filtered to that tool's available versions
- Allow prefixes ('3', '3.12') and 'latest' in addition to exact versions — matches how mise resolves version specs in .mise.toml

### Change tracking & write flow
- Visual distinction for modified/added/deleted rows is at Claude's discretion (color-coded, marker column, or both)
- Confirmation dialog and write-then-close behavior at Claude's discretion
- If user presses Esc with unsaved changes, show an "Unsaved changes. Discard? (y/n)" warning dialog before closing

### Claude's Discretion
- Env var entry method (two-step vs KEY=VALUE single-line)
- Visual style for change markers (color, icons, or both)
- Whether `w` write requires confirmation (and under what conditions)
- Whether editor stays open or closes after successful write
- Loading skeleton / spinner while parsing TOML

</decisions>

<specifics>
## Specific Ideas

- Table row layout for tools should look like: Name | Version | Status (compact, scannable)
- Registry fuzzy search for adding tools is the "current behaviour" — reuse, don't reinvent
- Version picker pre-filtered to selected tool makes edit flow smooth

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-inline-editor*
*Context gathered: 2026-02-23*
