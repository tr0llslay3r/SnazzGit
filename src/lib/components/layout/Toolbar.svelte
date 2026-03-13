<script lang="ts">
  import { repoInfo, isLoading, refreshAll, refreshCommits, refreshStatus, refreshRepo, closeRepo, loadRecentRepos, setupWatcher } from '$lib/stores/repo';
  import { showSearch, showThemePicker, showCloneDialog, showCredentialDialog, showCompareDialog, showReflog, compareRefs, pendingCredentialRequest, addToast } from '$lib/stores/ui';
  import { showContextMenu, type ContextMenuEntry } from '$lib/stores/contextmenu';
  import * as tauri from '$lib/utils/tauri';
  import { open } from '@tauri-apps/plugin-dialog';

  function isAuthError(e: unknown): boolean {
    return String(e).includes('Authentication failed');
  }

  async function openRepo() {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected) return;
      $isLoading = true;
      const info = await tauri.openRepository(selected as string);
      $repoInfo = info;
      await tauri.addRecentRepo(info.path, info.name);
      await loadRecentRepos();
      await Promise.all([refreshCommits(), refreshStatus()]);
      await setupWatcher(info.path);
      addToast(`Opened ${info.name}`, 'success');
    } catch (e) {
      addToast(`Failed to open repository: ${e}`, 'error');
    } finally {
      $isLoading = false;
    }
  }

  async function doFetch() {
    if (!$repoInfo || $repoInfo.remotes.length === 0) return;
    try {
      $isLoading = true;
      await tauri.fetchRemote($repoInfo.path, $repoInfo.remotes[0]);
      await refreshAll();
      addToast('Fetch complete', 'success');
    } catch (e) {
      if (isAuthError(e)) {
        $pendingCredentialRequest = { operation: 'fetch', remoteName: $repoInfo.remotes[0] };
        $showCredentialDialog = true;
      } else {
        addToast(`Fetch failed: ${e}`, 'error');
      }
    } finally {
      $isLoading = false;
    }
  }

  async function doPull() {
    if (!$repoInfo || $repoInfo.remotes.length === 0) return;
    try {
      $isLoading = true;
      const result = await tauri.pull($repoInfo.path, $repoInfo.remotes[0]);
      await refreshAll();
      addToast(`Pull: ${result}`, 'success');
    } catch (e) {
      if (isAuthError(e)) {
        $pendingCredentialRequest = { operation: 'pull', remoteName: $repoInfo.remotes[0] };
        $showCredentialDialog = true;
      } else {
        addToast(`Pull failed: ${e}`, 'error');
      }
    } finally {
      $isLoading = false;
    }
  }

  async function doPush() {
    if (!$repoInfo || $repoInfo.remotes.length === 0) return;
    try {
      $isLoading = true;
      await tauri.push($repoInfo.path, $repoInfo.remotes[0]);
      await refreshAll();
      addToast('Push complete', 'success');
    } catch (e) {
      if (isAuthError(e)) {
        $pendingCredentialRequest = { operation: 'push', remoteName: $repoInfo.remotes[0] };
        $showCredentialDialog = true;
      } else {
        addToast(`Push failed: ${e}`, 'error');
      }
    } finally {
      $isLoading = false;
    }
  }

  function onPushContext(e: MouseEvent) {
    e.preventDefault();
    if (!$repoInfo || $repoInfo.remotes.length === 0) return;
    const items: ContextMenuEntry[] = [
      { label: 'Force Push (--force-with-lease)', action: async () => {
        try {
          $isLoading = true;
          await tauri.forcePush($repoInfo!.path, $repoInfo!.remotes[0]);
          await refreshAll();
          addToast('Force push complete', 'success');
        } catch (e: unknown) {
          if (isAuthError(e)) {
            $pendingCredentialRequest = { operation: 'push', remoteName: $repoInfo!.remotes[0] };
            $showCredentialDialog = true;
          } else {
            addToast(`Force push failed: ${e}`, 'error');
          }
        } finally {
          $isLoading = false;
        }
      }},
    ];
    showContextMenu(e.clientX, e.clientY, items);
  }

  async function doRefresh() {
    try {
      await refreshAll();
    } catch (e) {
      addToast(`Refresh failed: ${e}`, 'error');
    }
  }
</script>

<nav class="toolbar">
  <div class="toolbar-group">
    <button class="toolbar-btn" onclick={openRepo} title="Open Repository">
      <svg viewBox="0 0 16 16" width="16" height="16"><path d="M1 3.5A1.5 1.5 0 0 1 2.5 2h3.879a1.5 1.5 0 0 1 1.06.44l1.122 1.12A1.5 1.5 0 0 0 9.62 4H13.5A1.5 1.5 0 0 1 15 5.5v7a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 1 12.5v-9z" fill="currentColor"/></svg>
      <span>Open</span>
    </button>
    <button class="toolbar-btn" onclick={() => $showCloneDialog = true} title="Clone Repository">
      <svg viewBox="0 0 16 16" width="16" height="16"><path d="M11.75 2.5a.75.75 0 0 1 .75.75V7h3.75a.75.75 0 0 1 0 1.5H12.5v3.75a.75.75 0 0 1-1.5 0V8.5H7.25a.75.75 0 0 1 0-1.5H11V3.25a.75.75 0 0 1 .75-.75z" fill="currentColor"/><path d="M2 1.75C2 .784 2.784 0 3.75 0h6.586c.464 0 .909.184 1.237.513l2.914 2.914c.329.328.513.773.513 1.237v9.586A1.75 1.75 0 0 1 13.25 16h-9.5A1.75 1.75 0 0 1 2 14.25V1.75zM3.5 1.75v12.5c0 .138.112.25.25.25h9.5a.25.25 0 0 0 .25-.25V5h-3.25A1.75 1.75 0 0 1 8.5 3.25V1.5H3.75a.25.25 0 0 0-.25.25zM10 1.667V3.25c0 .138.112.25.25.25h1.583L10 1.667z" fill="currentColor"/></svg>
      <span>Clone</span>
    </button>
    {#if $repoInfo}
      <button class="toolbar-btn" onclick={closeRepo} title="Close Repository">
        <svg viewBox="0 0 16 16" width="16" height="16"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06z" fill="currentColor"/></svg>
        <span>Close</span>
      </button>
    {/if}
  </div>

  {#if $repoInfo}
    <div class="toolbar-sep"></div>
    <div class="toolbar-group">
      <button class="toolbar-btn" onclick={doFetch} disabled={$isLoading || $repoInfo.remotes.length === 0} title="Fetch">
        <svg viewBox="0 0 16 16" width="16" height="16"><path d="M8 1a.75.75 0 0 1 .75.75v6.44l2.22-2.22a.75.75 0 1 1 1.06 1.06l-3.5 3.5a.75.75 0 0 1-1.06 0l-3.5-3.5a.75.75 0 0 1 1.06-1.06l2.22 2.22V1.75A.75.75 0 0 1 8 1z" fill="currentColor"/><path d="M2.5 10a.75.75 0 0 1 .75.75v2.5h9.5v-2.5a.75.75 0 0 1 1.5 0v2.5A1.75 1.75 0 0 1 12.5 15h-9A1.75 1.75 0 0 1 1.75 13.25v-2.5A.75.75 0 0 1 2.5 10z" fill="currentColor"/></svg>
        <span>Fetch</span>
      </button>
      <button class="toolbar-btn" onclick={doPull} disabled={$isLoading || $repoInfo.remotes.length === 0} title="Pull">
        <svg viewBox="0 0 16 16" width="16" height="16"><path d="M8 1a.75.75 0 0 1 .75.75v8.69l2.22-2.22a.75.75 0 1 1 1.06 1.06l-3.5 3.5a.75.75 0 0 1-1.06 0l-3.5-3.5a.75.75 0 1 1 1.06-1.06l2.22 2.22V1.75A.75.75 0 0 1 8 1z" fill="currentColor"/></svg>
        <span>Pull</span>
      </button>
      <button class="toolbar-btn" onclick={doPush} oncontextmenu={onPushContext} disabled={$isLoading || $repoInfo.remotes.length === 0} title="Push (right-click for force push)">
        <svg viewBox="0 0 16 16" width="16" height="16"><path d="M8 15a.75.75 0 0 1-.75-.75V5.56L5.03 7.78a.75.75 0 0 1-1.06-1.06l3.5-3.5a.75.75 0 0 1 1.06 0l3.5 3.5a.75.75 0 1 1-1.06 1.06L8.75 5.56v8.69A.75.75 0 0 1 8 15z" fill="currentColor"/></svg>
        <span>Push</span>
      </button>
    </div>

    <div class="toolbar-sep"></div>
    <div class="toolbar-group">
      <button class="toolbar-btn" onclick={() => $showReflog = !$showReflog} class:active={$showReflog} title="Reflog">
        <svg viewBox="0 0 16 16" width="16" height="16"><path d="M8 3a5 5 0 1 0 4.546 2.914.5.5 0 1 1 .908-.418A6 6 0 1 1 8 2v1z" fill="currentColor"/><path d="M8 5v3.5l2.5 1.5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round"/></svg>
        <span>Reflog</span>
      </button>
      <button class="toolbar-btn" onclick={() => $showCompareDialog = true} title="Compare refs (Ctrl+D)">
        <svg viewBox="0 0 16 16" width="16" height="16"><path d="M5.22 14.78a.75.75 0 0 0 1.06-1.06L4.56 12h8.69a.75.75 0 0 0 0-1.5H4.56l1.72-1.72a.75.75 0 0 0-1.06-1.06l-3 3a.75.75 0 0 0 0 1.06l3 3zM10.78 1.22a.75.75 0 0 0-1.06 1.06L11.44 4H2.75a.75.75 0 0 0 0 1.5h8.69l-1.72 1.72a.75.75 0 1 0 1.06 1.06l3-3a.75.75 0 0 0 0-1.06l-3-3z" fill="currentColor"/></svg>
        <span>Compare</span>
      </button>
      <button class="toolbar-btn" onclick={doRefresh} disabled={$isLoading} title="Refresh">
        <svg viewBox="0 0 16 16" width="16" height="16"><path d="M8 3a5 5 0 1 0 4.546 2.914.5.5 0 1 1 .908-.418A6 6 0 1 1 8 2v1z" fill="currentColor"/><path d="M8 1v4l3-2-3-2z" fill="currentColor"/></svg>
        <span>Refresh</span>
      </button>
    </div>
  {/if}

  <div class="toolbar-spacer"></div>

  <div class="toolbar-group">
    <button class="toolbar-btn" onclick={() => $showSearch = true} title="Search (Ctrl+K)">
      <svg viewBox="0 0 16 16" width="16" height="16"><path d="M11.5 7a4.5 4.5 0 1 1-9 0 4.5 4.5 0 0 1 9 0zm-.82 4.74a6 6 0 1 1 1.06-1.06l3.04 3.04a.75.75 0 1 1-1.06 1.06l-3.04-3.04z" fill="currentColor"/></svg>
    </button>
    <button class="toolbar-btn" onclick={() => $showThemePicker = !$showThemePicker} title="Theme">
      <svg viewBox="0 0 16 16" width="16" height="16"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 13A6 6 0 0 1 8 2a6 6 0 0 1 0 12z" fill="currentColor"/><circle cx="8" cy="8" r="3" fill="currentColor"/></svg>
    </button>
  </div>

  {#if $isLoading}
    <div class="loading-bar"></div>
  {/if}
</nav>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    position: relative;
    flex-shrink: 0;
    height: 42px;
  }
  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border: none;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: 6px;
    font-size: 12px;
    white-space: nowrap;
    height: 30px;
    transition: all 0.15s;
  }
  .toolbar-btn:hover:not(:disabled) {
    background: var(--bg-surface);
    color: var(--text-primary);
  }
  .toolbar-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .toolbar-btn.active {
    background: var(--bg-surface);
    color: var(--accent);
  }
  .toolbar-sep {
    width: 1px;
    height: 20px;
    background: var(--border);
    margin: 0 4px;
  }
  .toolbar-spacer {
    flex: 1;
  }
  .loading-bar {
    position: absolute;
    bottom: 0;
    left: 0;
    height: 2px;
    background: var(--accent);
    animation: loading 1.5s ease-in-out infinite;
  }
  @keyframes loading {
    0% { width: 0; left: 0; }
    50% { width: 60%; left: 20%; }
    100% { width: 0; left: 100%; }
  }
</style>
