# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is SnazzGit?

A cross-platform Git GUI client built with Tauri 2 (Rust backend) + Svelte 5 (frontend) + Tailwind CSS 4. Uses git2 for all git operations - no shell git commands.

## Commands

```bash
# Development (starts both Rust backend and Vite dev server)
cargo tauri dev

# Production build (outputs .deb, .rpm in src-tauri/target/release/bundle/)
cargo tauri build

# Frontend only
npm run build          # Vite build
npm run check          # svelte-check + TypeScript

# Rust only
cd src-tauri && cargo build
cd src-tauri && cargo clippy
```

## Testing

Run Rust tests:
```bash
cd src-tauri && cargo test
```

Run with coverage:
```bash
cd src-tauri && cargo llvm-cov --lib
```

Run TypeScript tests:
```bash
npm test
```

### Rust test structure

- **`git/`** — Unit tests embedded in each module via `#[cfg(test)]`. Use `tempfile::TempDir` + `git2::Repository::init()` for isolated repos.
- **`commands/`** — Each command file has a `#[cfg(test)]` block. Command functions are plain `async fn` and can be called directly with `#[tokio::test]` — no Tauri runtime needed.
- **`commands/remotes.rs`** and **`commands/clone.rs`** — Not unit-tested (require `AppHandle` for progress events).

When writing new command tests, use the same `init_repo_with_commit()` helper pattern already present in each file.

## Architecture

### IPC Boundary (the critical seam)

The frontend calls Rust via Tauri's `invoke()`. The full contract is defined in three places that must stay in sync:

1. **Rust command handlers** — `src-tauri/src/commands/*.rs` — `#[tauri::command]` functions registered in `lib.rs`
2. **TypeScript IPC bindings** — `src/lib/utils/tauri.ts` — typed `invoke()` wrappers
3. **Shared types** — Rust: `src-tauri/src/git/types.rs`, TS: `src/lib/types/index.ts`

When adding/changing a command: update all three layers. Command names use `snake_case` in Rust, matching the invoke string in TS.

### Rust Backend (`src-tauri/src/`)

Two-layer separation:
- **`git/`** — Pure git2 logic, no Tauri dependency. Each file maps to a domain (commit, branch, diff, status, stash, blame, search, remote, graph, rebase, reflog, tag, watcher). Error type in `git/error.rs` uses `thiserror`.
- **`commands/`** — Thin Tauri IPC wrappers that call into `git/`. Every command uses `tokio::task::spawn_blocking` because `git2::Repository` is not `Send`.
- **`theme.rs`** — User theme persistence at `~/.config/snazzgit/themes/`.

Key pattern: `git2::Repository` is opened fresh in each `spawn_blocking` closure (the repo path is passed as a String, not stored).

### Frontend (`src/lib/`)

- **`stores/repo.ts`** — Central state. Exports `refreshAll()`, `refreshRepo()`, `refreshCommits()`, `refreshStatus()`, `refreshStashes()`, `setupWatcher()`. All state-modifying UI actions should call the appropriate refresh after completion. The file watcher calls only `refreshStatus()` (not `refreshRepo()`) to avoid full UI re-renders.
- **`stores/theme.ts`** — Theme store. CSS variables are injected at runtime; Tailwind's `@theme` directive maps them.
- **`stores/ui.ts`** — UI state (selected commit, active panel, etc.)
- **`stores/contextmenu.ts`** — Context menu state. Menu entries support submenus via `children` property.
- **`utils/tauri.ts`** — All IPC calls. Components never call `invoke()` directly.
- **`components/`** — Organized by domain: `commit/`, `branch/`, `diff/`, `staging/`, `theme/`, `layout/`, `shared/`.

### Reactivity Pitfall

In `$effect` blocks, avoid depending on full store objects (e.g., `$repoInfo`) when only a primitive field is needed (e.g., the repo path). Use `$derived` to extract the primitive first. Otherwise, the watcher's `refreshStatus()` or any store update that creates a new object reference will re-trigger the effect and cause race conditions with in-flight async calls. See `DiffView.svelte` `repoPath` derived for the pattern.

### Graph Rendering

Commit graph is computed in Rust (`git/graph.rs`), returns `GraphRow[]` with column positions and edges. Frontend renders as SVG. CommitList uses virtual scrolling with 28px row height.

### SvelteKit Config

Uses `adapter-static` with `fallback: 'index.html'`, SSR disabled. This is a single-page Tauri app, not a traditional web app.

## Conventions

- Svelte 5 runes (`$state`, `$derived`, `$effect`) — not legacy `$:` reactive statements
- Tailwind CSS 4 (not v3) — uses `@theme` directive for CSS variable integration
- All Tauri commands are async and return `Result<T, String>` on the command layer
- Theme colors are CSS custom properties (`--color-*`), defined in `src/lib/themes/*.ts`
- The staging area is always visible in the bottom pane when no commit is selected (no toggle button)
- Keyboard shortcuts: Ctrl+K (search), Ctrl+B (branch), Ctrl+D (compare), Ctrl+G (stash), Ctrl+Shift+S (stage all), Ctrl+Shift+U (unstage all), Escape (close/deselect)
