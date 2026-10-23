# SnazzGit: Missing Features — Low-Hanging Fruit First

## Context

SnazzGit is already a full-featured Git GUI with commits, branches, tags, stashes, remotes, staging (hunk-level), diffs, blame, search, reflog, cherry-pick, rebase, conflict resolution, clone, file history, compare refs, themes, and recent repos. This plan identifies the most impactful features still missing, ordered from easiest to hardest.

---

## Tier 1: Trivial (< 1 hour each, minimal risk)

### 1. Branch Ahead/Behind Upstream Count
**Impact: HIGH** — Every Git GUI shows this; users need it to know when to push/pull.

- **Rust**: Add `ahead: Option<u32>`, `behind: Option<u32>` to `BranchInfo` in `types.rs`. In `repository.rs` where branches are loaded, call `repo.graph_ahead_behind(local_oid, upstream_oid)` for branches with an upstream.
- **TS types**: Add `ahead?: number`, `behind?: number` to `BranchInfo` in `types/index.ts`
- **Sidebar UI**: Show `↑2 ↓3` badges next to branch names in `Sidebar.svelte`
- **StatusBar**: Show ahead/behind for current branch in `StatusBar.svelte`
- **Files**: `src-tauri/src/git/types.rs`, `src-tauri/src/git/repository.rs`, `src/lib/types/index.ts`, `src/lib/components/layout/Sidebar.svelte`, `src/lib/components/layout/StatusBar.svelte`

### 2. Create Branch from Arbitrary Commit
**Impact: MEDIUM** — There's already a TODO and a "Create Branch Here" context menu entry.

- **Rust**: Change `create_branch()` in `git/branch.rs` to accept an optional `commit_id: Option<&str>` parameter. If provided, use `repo.find_commit(Oid::from_str(id)?)` instead of `repo.head()`.
- **Command**: Update `commands/branches.rs` `create_branch` to pass the commit ID through.
- **TS**: Update `createBranch()` in `tauri.ts` to accept optional `commitId`.
- **UI**: The "Create Branch Here" context menu in `CommitList.svelte` should open BranchDialog pre-filled with the commit.
- **Files**: `src-tauri/src/git/branch.rs`, `src-tauri/src/commands/branches.rs`, `src/lib/utils/tauri.ts`, `src/lib/components/commit/CommitList.svelte`, `src/lib/components/shared/BranchDialog.svelte`

### 3. Quick "Undo Last Commit" (Soft Reset HEAD~1)
**Impact: HIGH** — Extremely common action, already have `reset_to_commit` infrastructure.

- **No new Rust code needed** — reuse existing `reset_to_commit` with soft mode
- **UI**: Add "Undo Last Commit" button/context menu entry in toolbar or commit list (for HEAD commit)
- **Files**: `src/lib/components/layout/Toolbar.svelte` or `src/lib/components/commit/CommitList.svelte`

### 4. Open File in System File Manager / Default Editor
**Impact: MEDIUM** — Quality-of-life for navigating between GUI and editor.

- **Rust**: Use Tauri's `tauri::api::shell::open()` or `opener` crate. Add two commands: `open_in_editor(path)` and `reveal_in_file_manager(path)`.
- **UI**: Add context menu entries on files in staging area and diff view.
- **Files**: `src-tauri/src/commands/` (new or in repo.rs), `src/lib/utils/tauri.ts`, `src/lib/components/staging/FileEntry.svelte`

### 5. Open Terminal at Repo Path
**Impact: MEDIUM** — Common in Git GUIs (GitKraken, Fork, etc.).

- **Rust**: Use `std::process::Command` to open platform-specific terminal at the repo path.
- **UI**: Add toolbar button or menu entry.
- **Files**: `src-tauri/src/commands/repo.rs`, `src/lib/utils/tauri.ts`, `src/lib/components/layout/Toolbar.svelte`

---

## Tier 2: Easy (1–3 hours each)

### 6. Stash Content Preview
**Impact: HIGH** — Users want to see what's in a stash before applying/popping.

- **Rust**: Add `stash_diff(path, index)` in `git/stash.rs` — get stash commit via `repo.stash_foreach()`, then diff it against its parent. Return `Vec<DiffFile>` (reuse existing diff types).
- **Command + TS binding**: New `get_stash_diff` command.
- **UI**: When clicking a stash in sidebar, show its diff in the bottom pane (similar to commit detail).
- **Files**: `src-tauri/src/git/stash.rs`, `src-tauri/src/commands/stash.rs`, `src/lib/utils/tauri.ts`, `src/lib/components/layout/Sidebar.svelte`, possibly new `StashDetail.svelte`

### 7. Branch Filter/Search in Sidebar
**Impact: HIGH** for repos with many branches.

- **No backend changes** — filter purely in frontend.
- **UI**: Add a small search/filter input at top of the Branches section in Sidebar. Filter the TreeView items client-side.
- **Files**: `src/lib/components/layout/Sidebar.svelte`

### 8. Word-Level (Inline) Diff Highlighting
**Impact: HIGH** — Shows exactly which characters changed within a line. Most Git GUIs have this.

- **Frontend-only**: Compare adjacent addition/deletion line pairs character-by-character. Wrap changed segments in `<span>` with highlight class.
- **Algorithm**: Simple longest-common-subsequence or word-boundary diff between paired add/delete lines.
- **Files**: `src/lib/components/diff/DiffLine.svelte` (add inline highlight spans)

### 9. Auto-Fetch on Interval
**Impact: MEDIUM** — Keeps the UI up-to-date with remote changes.

- **Frontend-only**: `setInterval` in the repo store that calls `fetchRemote()` every N minutes (configurable, default 5 min). Add a setting toggle.
- **Files**: `src/lib/stores/repo.ts`, possibly a settings store

---

## Tier 3: Medium (3–6 hours each)

### 10. Git Config Viewer/Editor
**Impact: MEDIUM** — View and edit user.name, user.email, remote URLs, etc.

- **Rust**: Use `repo.config()` to read/write config entries. New commands: `get_config_entries()`, `set_config_entry(key, value)`.
- **UI**: New settings/config dialog accessible from toolbar.
- **Files**: New `src-tauri/src/git/config.rs`, new `src-tauri/src/commands/config.rs`, new `ConfigDialog.svelte`

### 11. Patch Export/Import
**Impact: LOW-MEDIUM** — Useful for email-based workflows.

- **Rust**: Use `git2::Diff::format_email()` for export. For import, apply patch via `git2::apply`.
- **UI**: Context menu on commits "Export as Patch", toolbar "Apply Patch" button.
- **Files**: New functions in `git/diff.rs`, new command handlers, UI buttons

### 12. Submodule Support (Read-Only Display)
**Impact: MEDIUM** for repos that use submodules.

- **Rust**: Use `repo.submodules()` to list submodules with path, URL, status.
- **UI**: New "Submodules" section in sidebar.
- **Files**: New `src-tauri/src/git/submodule.rs`, sidebar update

---

## Tier 4: Hard (6+ hours, higher risk)

### 13. Line-Level Staging (Finer than Hunk)
### 14. Interactive Rebase UI
### 15. GPG Commit Signing

---

## Recommended Implementation Order

For maximum impact with minimal effort, implement in this order:
1. **Ahead/behind count** (#1) — highest value-to-effort ratio
2. **Branch from commit** (#2) — fixes an existing TODO
3. **Undo last commit** (#3) — zero new backend code
4. **Stash preview** (#6) — frequently requested in Git GUIs
5. **Branch filter** (#7) — pure frontend, easy win
6. **Word-level diff** (#8) — big UX improvement, frontend-only
7. **Open file externally** (#4) — QoL
8. **Open terminal** (#5) — QoL
9. **Auto-fetch** (#9) — keeps UI fresh

## Verification

For each feature:
- Run `cd src-tauri && cargo test` after any Rust changes
- Run `npm run check` after any TS/Svelte changes
- Run `cd src-tauri && cargo clippy` to check for warnings
- Manual test via `cargo tauri dev`
