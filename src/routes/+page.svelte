<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import Toolbar from '$lib/components/layout/Toolbar.svelte';
  import StatusBar from '$lib/components/layout/StatusBar.svelte';
  import CommitList from '$lib/components/commit/CommitList.svelte';
  import CommitDetail from '$lib/components/commit/CommitDetail.svelte';
  import FileHistory from '$lib/components/commit/FileHistory.svelte';
  import ReflogView from '$lib/components/commit/ReflogView.svelte';
  import DiffView from '$lib/components/diff/DiffView.svelte';
  import StagingArea from '$lib/components/staging/StagingArea.svelte';
  import ThemePicker from '$lib/components/theme/ThemePicker.svelte';
  import BranchDialog from '$lib/components/branch/BranchDialog.svelte';
  import MergeDialog from '$lib/components/branch/MergeDialog.svelte';
  import CheckoutRemoteDialog from '$lib/components/branch/CheckoutRemoteDialog.svelte';
  import StashDialog from '$lib/components/shared/StashDialog.svelte';
  import TagDialog from '$lib/components/shared/TagDialog.svelte';
  import CredentialDialog from '$lib/components/shared/CredentialDialog.svelte';
  import CloneDialog from '$lib/components/shared/CloneDialog.svelte';
  import CompareDialog from '$lib/components/shared/CompareDialog.svelte';
  import AddRemoteDialog from '$lib/components/shared/AddRemoteDialog.svelte';
  import CompareView from '$lib/components/diff/CompareView.svelte';
  import { repoInfo, commits, recentRepos, isLoading, refreshCommits, refreshStatus, loadRecentRepos, setupWatcher } from '$lib/stores/repo';
  import { selectedCommit, showCloneDialog, addToast, fileHistoryPath, compareRefs, showReflog } from '$lib/stores/ui';
  import * as tauri from '$lib/utils/tauri';

  async function openRecent(path: string) {
    try {
      $isLoading = true;
      const info = await tauri.openRepository(path);
      $repoInfo = info;
      await tauri.addRecentRepo(info.path, info.name);
      await loadRecentRepos();
      await Promise.all([refreshCommits(), refreshStatus()]);
      await setupWatcher(info.path);
    } catch (e) {
      addToast(`Failed to open repository: ${e}`, 'error');
    } finally {
      $isLoading = false;
    }
  }

  onMount(() => {
    let unlistenDrop: (() => void) | undefined;
    getCurrentWindow().onDragDropEvent(async (event) => {
      if (event.payload.type === 'drop') {
        const paths = event.payload.paths;
        if (paths.length > 0) {
          try {
            $isLoading = true;
            const info = await tauri.openRepository(paths[0]);
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
      }
    }).then(fn => { unlistenDrop = fn; });

    return () => { unlistenDrop?.(); };
  });

  async function removeRecent(path: string) {
    try {
      await tauri.removeRecentRepo(path);
      await loadRecentRepos();
    } catch (e) {
      addToast(`Failed to remove: ${e}`, 'error');
    }
  }

  const LAYOUT_KEY = 'snazzgit-layout';

  interface Layout { sidebarWidth: number; bottomHeight: number; stagingWidth: number }

  function loadLayout(): Layout {
    try {
      const saved = localStorage.getItem(LAYOUT_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        return {
          sidebarWidth: parsed.sidebarWidth ?? 240,
          bottomHeight: parsed.bottomHeight ?? 300,
          stagingWidth: parsed.stagingWidth ?? 300,
        };
      }
    } catch { /* ignore */ }
    return { sidebarWidth: 240, bottomHeight: 300, stagingWidth: 300 };
  }

  function saveLayout() {
    try {
      localStorage.setItem(LAYOUT_KEY, JSON.stringify({
        sidebarWidth, bottomHeight, stagingWidth,
      }));
    } catch { /* ignore */ }
  }

  const initial = loadLayout();
  let sidebarWidth = $state(initial.sidebarWidth);
  let draggingSidebar = $state(false);
  let bottomHeight = $state(initial.bottomHeight);
  let draggingBottom = $state(false);
  let stagingWidth = $state(initial.stagingWidth);
  let draggingStaging = $state(false);
  let mainArea: HTMLDivElement;

  function onSidebarDrag(e: MouseEvent) {
    e.preventDefault();
    draggingSidebar = true;
    const onMove = (e: MouseEvent) => {
      sidebarWidth = Math.max(180, Math.min(400, e.clientX));
    };
    const onUp = () => {
      draggingSidebar = false;
      saveLayout();
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  function onStagingDrag(e: MouseEvent) {
    e.preventDefault();
    draggingStaging = true;
    const onMove = (e: MouseEvent) => {
      const offset = sidebarWidth + 3; // sidebar + sidebar divider
      stagingWidth = Math.max(180, Math.min(600, e.clientX - offset));
    };
    const onUp = () => {
      draggingStaging = false;
      saveLayout();
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  function onBottomDrag(e: MouseEvent) {
    e.preventDefault();
    draggingBottom = true;
    const onMove = (e: MouseEvent) => {
      if (!mainArea) return;
      const rect = mainArea.getBoundingClientRect();
      bottomHeight = Math.max(150, Math.min(rect.height - 200, rect.bottom - e.clientY));
    };
    const onUp = () => {
      draggingBottom = false;
      saveLayout();
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }
</script>

<div class="app-shell">
  {#if $repoInfo}
    <div class="app-sidebar" style="width: {sidebarWidth}px">
      <Sidebar />
    </div>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="sidebar-divider"
      class:active={draggingSidebar}
      onmousedown={onSidebarDrag}
      role="separator"
    ></div>
  {/if}
  <div class="app-main">
    <Toolbar />
    <div class="main-content" bind:this={mainArea}>
      {#if $repoInfo}
        <div class="top-pane" style="flex: 1">
          <CommitList />
        </div>
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div
          class="bottom-divider"
          class:active={draggingBottom}
          onmousedown={onBottomDrag}
          role="separator"
        ></div>
        <div class="bottom-pane" style="height: {bottomHeight}px">
          {#if $showReflog}
            <ReflogView />
          {:else if $compareRefs}
            <CompareView />
          {:else if $fileHistoryPath && !$selectedCommit}
            <div class="bottom-split">
              <div class="staging-col" style="width: {stagingWidth}px">
                <FileHistory />
              </div>
              <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
              <div
                class="staging-divider"
                class:active={draggingStaging}
                onmousedown={onStagingDrag}
                role="separator"
              ></div>
              <div class="diff-col">
                <DiffView />
              </div>
            </div>
          {:else if $selectedCommit}
            <CommitDetail />
          {:else}
            <div class="bottom-split">
              <div class="staging-col" style="width: {stagingWidth}px">
                <StagingArea />
              </div>
              <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
              <div
                class="staging-divider"
                class:active={draggingStaging}
                onmousedown={onStagingDrag}
                role="separator"
              ></div>
              <div class="diff-col">
                <DiffView showHunkActions />
              </div>
            </div>
          {/if}
        </div>
      {:else}
        <div class="welcome">
          <div class="welcome-content">
            <h1 class="welcome-title">SnazzGit</h1>
            {#if $recentRepos.length > 0}
              <p class="welcome-subtitle">Recent Repositories</p>
              <div class="recent-list">
                {#each $recentRepos as repo}
                  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
                  <div class="recent-item" onclick={() => openRecent(repo.path)}>
                    <div class="recent-icon">
                      <svg viewBox="0 0 16 16" width="20" height="20"><path d="M2 1.75C2 .784 2.784 0 3.75 0h6.586c.464 0 .909.184 1.237.513l2.914 2.914c.329.328.513.773.513 1.237v9.586A1.75 1.75 0 0 1 13.25 16h-9.5A1.75 1.75 0 0 1 2 14.25V1.75z" fill="currentColor"/></svg>
                    </div>
                    <div class="recent-info">
                      <span class="recent-name">{repo.name}</span>
                      <span class="recent-path">{repo.path}</span>
                    </div>
                    <button
                      class="recent-remove"
                      title="Remove from recent"
                      onclick={(e) => { e.stopPropagation(); removeRecent(repo.path); }}
                    >
                      <svg viewBox="0 0 16 16" width="14" height="14"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06z" fill="currentColor"/></svg>
                    </button>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="welcome-subtitle">Open a repository to get started</p>
            {/if}
            <p class="welcome-hint">Use the <strong>Open</strong> button in the toolbar to browse for a repository</p>
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
            <p class="welcome-hint">or <span class="clone-link" onclick={() => $showCloneDialog = true}>clone a repository</span></p>
          </div>
        </div>
      {/if}
    </div>
    <StatusBar />
  </div>
</div>

<ThemePicker />
<BranchDialog />
<MergeDialog />
<CheckoutRemoteDialog />
<StashDialog />
<TagDialog />
<CredentialDialog />
<CloneDialog />
<CompareDialog />
<AddRemoteDialog />

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--bg-primary);
  }
  .app-sidebar {
    flex-shrink: 0;
    overflow: hidden;
  }
  .sidebar-divider {
    width: 3px;
    cursor: col-resize;
    background: var(--border);
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .sidebar-divider:hover,
  .sidebar-divider.active {
    background: var(--accent);
  }
  .app-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .top-pane {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .bottom-divider {
    height: 3px;
    cursor: row-resize;
    background: var(--border);
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .bottom-divider:hover,
  .bottom-divider.active {
    background: var(--accent);
  }
  .bottom-pane {
    flex-shrink: 0;
    overflow: hidden;
    border-top: 1px solid var(--border);
  }
  .bottom-split {
    display: flex;
    height: 100%;
  }
  .staging-col {
    flex-shrink: 0;
    overflow: hidden;
  }
  .staging-divider {
    width: 3px;
    cursor: col-resize;
    background: var(--border);
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .staging-divider:hover,
  .staging-divider.active {
    background: var(--accent);
  }
  .diff-col {
    flex: 1;
    overflow: hidden;
  }
  .empty-bottom {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }
  .welcome {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .welcome-content {
    text-align: center;
  }
  .welcome-title {
    font-size: 48px;
    font-weight: 700;
    color: var(--accent);
    margin: 0 0 8px;
  }
  .welcome-subtitle {
    font-size: 18px;
    color: var(--text-primary);
    margin: 0 0 12px;
  }
  .welcome-hint {
    font-size: 14px;
    color: var(--text-secondary);
    margin: 0;
  }
  .recent-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-width: 500px;
    margin: 16px auto 20px;
    max-height: 400px;
    overflow-y: auto;
  }
  .recent-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    color: var(--text-primary);
    transition: all 0.15s;
  }
  .recent-item:hover {
    background: var(--bg-surface);
    border-color: var(--accent);
  }
  .recent-icon {
    flex-shrink: 0;
    color: var(--accent);
    display: flex;
    align-items: center;
  }
  .recent-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .recent-name {
    font-size: 14px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .recent-path {
    font-size: 11px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .recent-remove {
    flex-shrink: 0;
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    opacity: 0;
    transition: all 0.15s;
  }
  .recent-item:hover .recent-remove {
    opacity: 1;
  }
  .recent-remove:hover {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }
  .clone-link {
    color: var(--accent);
    cursor: pointer;
    text-decoration: underline;
  }
  .clone-link:hover {
    opacity: 0.8;
  }
</style>
