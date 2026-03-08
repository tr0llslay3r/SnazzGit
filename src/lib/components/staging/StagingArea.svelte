<script lang="ts">
  import FileEntry from './FileEntry.svelte';
  import { repoInfo, workingStatus, refreshAll, refreshStatus } from '$lib/stores/repo';
  import { selectedFile, selectedFileStaged, addToast } from '$lib/stores/ui';
  import { showContextMenu, type ContextMenuEntry } from '$lib/stores/contextmenu';
  import * as tauri from '$lib/utils/tauri';

  let commitMessage = $state('');
  let isAmend = $state(false);
  let unstagedRatio = $state(50);
  let dragging = $state(false);
  let containerEl: HTMLDivElement | undefined = $state();

  function onDividerDrag(e: MouseEvent) {
    e.preventDefault();
    dragging = true;
    const onMove = (e: MouseEvent) => {
      if (!containerEl) return;
      const rect = containerEl.getBoundingClientRect();
      const commitPanel = containerEl.querySelector('.commit-panel') as HTMLElement;
      const commitHeight = commitPanel?.offsetHeight ?? 0;
      const available = rect.height - commitHeight;
      if (available <= 0) return;
      const y = e.clientY - rect.top;
      unstagedRatio = Math.max(15, Math.min(85, (y / available) * 100));
    };
    const onUp = () => {
      dragging = false;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  async function stage(filePath: string) {
    if (!$repoInfo) return;
    try {
      await tauri.stageFile($repoInfo.path, filePath);
      $workingStatus = await tauri.getStatus($repoInfo.path);
    } catch (e) {
      addToast(`Failed to stage: ${e}`, 'error');
    }
  }

  async function unstage(filePath: string) {
    if (!$repoInfo) return;
    try {
      await tauri.unstageFile($repoInfo.path, filePath);
      $workingStatus = await tauri.getStatus($repoInfo.path);
    } catch (e) {
      addToast(`Failed to unstage: ${e}`, 'error');
    }
  }

  async function stageAllFiles() {
    if (!$repoInfo) return;
    try {
      await tauri.stageAll($repoInfo.path);
      $workingStatus = await tauri.getStatus($repoInfo.path);
    } catch (e) {
      addToast(`Failed to stage all: ${e}`, 'error');
    }
  }

  async function unstageAllFiles() {
    if (!$repoInfo) return;
    try {
      await tauri.unstageAll($repoInfo.path);
      $workingStatus = await tauri.getStatus($repoInfo.path);
    } catch (e) {
      addToast(`Failed to unstage all: ${e}`, 'error');
    }
  }

  async function discardFileChanges(filePath: string) {
    if (!$repoInfo) return;
    try {
      await tauri.discardFile($repoInfo.path, filePath);
      $workingStatus = await tauri.getStatus($repoInfo.path);
    } catch (e) {
      addToast(`Failed to discard: ${e}`, 'error');
    }
  }

  async function doCommit() {
    if (!$repoInfo || !commitMessage.trim()) return;
    try {
      await tauri.createCommit($repoInfo.path, commitMessage, isAmend);
      commitMessage = '';
      isAmend = false;
      await refreshAll();
      addToast('Commit created', 'success');
    } catch (e) {
      addToast(`Commit failed: ${e}`, 'error');
    }
  }

  async function ignorePattern(pattern: string) {
    if (!$repoInfo) return;
    try {
      await tauri.addToGitignore($repoInfo.path, pattern);
      $workingStatus = await tauri.getStatus($repoInfo.path);
      addToast(`Added '${pattern}' to .gitignore`, 'success');
    } catch (e) {
      addToast(`Failed to update .gitignore: ${e}`, 'error');
    }
  }

  function getIgnoreItems(filePath: string): ContextMenuEntry[] {
    const parts = filePath.split('/');
    const fileName = parts[parts.length - 1];
    const ext = fileName.includes('.') ? fileName.substring(fileName.lastIndexOf('.')) : null;
    const items: ContextMenuEntry[] = [];

    // Ignore exact file
    items.push({ label: `Ignore file`, action: () => ignorePattern(filePath) });

    // Ignore by extension
    if (ext) {
      items.push({ label: `Ignore *${ext}`, action: () => ignorePattern(`*${ext}`) });
    }

    // Ignore directory submenu (all ancestor directories, deepest first)
    if (parts.length > 1) {
      const dirItems: ContextMenuEntry[] = [];
      for (let i = parts.length - 1; i >= 1; i--) {
        const dir = parts.slice(0, i).join('/') + '/';
        dirItems.push({ label: dir, action: () => ignorePattern(dir) });
      }
      items.push({ label: 'Ignore directory', children: dirItems });
    }

    return items;
  }

  function onUnstagedContext(filePath: string, status: string, e: MouseEvent) {
    const items: ContextMenuEntry[] = [
      { label: 'Stage', action: () => stage(filePath) },
    ];
    if (status !== 'untracked' && status !== 'New') {
      items.push({ label: 'Discard Changes', danger: true, action: () => discardFileChanges(filePath) });
    }
    items.push({ separator: true });
    items.push(...getIgnoreItems(filePath));
    items.push({ separator: true });
    items.push({ label: 'Copy Path', action: () => navigator.clipboard.writeText(filePath) });
    showContextMenu(e.clientX, e.clientY, items);
  }

  function onStagedContext(filePath: string, e: MouseEvent) {
    const items: ContextMenuEntry[] = [
      { label: 'Unstage', action: () => unstage(filePath) },
      { separator: true },
      { label: 'Copy Path', action: () => navigator.clipboard.writeText(filePath) },
    ];
    showContextMenu(e.clientX, e.clientY, items);
  }

  function onCommitKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      doCommit();
    }
  }
</script>

<div class="staging-area" bind:this={containerEl}>
  {#if $workingStatus}
    <div class="staging-section" style="flex: {unstagedRatio}">
      <div class="staging-header">
        <span>Unstaged Changes ({$workingStatus.unstaged.length + $workingStatus.untracked.length})</span>
        <button class="staging-action" onclick={stageAllFiles} title="Stage all">+All</button>
      </div>
      <div class="file-list">
        {#each $workingStatus.unstaged as file (file.path)}
          <FileEntry
            path={file.path}
            status={file.status}
            selected={$selectedFile === file.path && !$selectedFileStaged}
            onSelect={() => { $selectedFile = file.path; $selectedFileStaged = false; }}
            onAction={() => stage(file.path)}
            actionLabel="Stage"
            actionIcon="+"
            onContextMenu={(e) => onUnstagedContext(file.path, file.status, e)}
          />
        {/each}
        {#each $workingStatus.untracked as path (path)}
          <FileEntry
            {path}
            status="untracked"
            selected={$selectedFile === path && !$selectedFileStaged}
            onSelect={() => { $selectedFile = path; $selectedFileStaged = false; }}
            onAction={() => stage(path)}
            actionLabel="Stage"
            actionIcon="+"
            onContextMenu={(e) => onUnstagedContext(path, 'untracked', e)}
          />
        {/each}
      </div>
    </div>

    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="staging-divider"
      class:active={dragging}
      onmousedown={onDividerDrag}
      role="separator"
    ></div>

    <div class="staging-section" style="flex: {100 - unstagedRatio}">
      <div class="staging-header">
        <span>Staged Changes ({$workingStatus.staged.length})</span>
        <button class="staging-action" onclick={unstageAllFiles} title="Unstage all">-All</button>
      </div>
      <div class="file-list">
        {#each $workingStatus.staged as file (file.path)}
          <FileEntry
            path={file.path}
            status={file.status}
            selected={$selectedFile === file.path && $selectedFileStaged}
            onSelect={() => { $selectedFile = file.path; $selectedFileStaged = true; }}
            onAction={() => unstage(file.path)}
            actionLabel="Unstage"
            actionIcon="-"
            onContextMenu={(e) => onStagedContext(file.path, e)}
          />
        {/each}
      </div>
    </div>

    <div class="commit-panel">
      <textarea
        class="commit-input"
        placeholder="Commit message..."
        bind:value={commitMessage}
        onkeydown={onCommitKeydown}
        rows="3"
      ></textarea>
      <div class="commit-actions">
        <label class="amend-check">
          <input type="checkbox" bind:checked={isAmend} />
          <span>Amend</span>
        </label>
        <button
          class="commit-btn"
          onclick={doCommit}
          disabled={!commitMessage.trim() || $workingStatus.staged.length === 0}
        >
          Commit ({$workingStatus.staged.length})
        </button>
      </div>
    </div>
  {:else}
    <div class="no-status">
      <p>Open a repository to see changes</p>
    </div>
  {/if}
</div>

<style>
  .staging-area {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .staging-section {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }
  .staging-divider {
    height: 3px;
    cursor: row-resize;
    background: var(--border);
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .staging-divider:hover,
  .staging-divider.active {
    background: var(--accent);
  }
  .staging-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-secondary);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .staging-action {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
  }
  .staging-action:hover {
    color: var(--text-primary);
    border-color: var(--accent);
  }
  .file-list {
    overflow-y: auto;
    flex: 1;
  }
  .commit-panel {
    padding: 8px;
    border-top: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }
  .commit-input {
    width: 100%;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    padding: 8px;
    font-size: 12px;
    font-family: inherit;
    resize: vertical;
    outline: none;
    box-sizing: border-box;
  }
  .commit-input:focus {
    border-color: var(--accent);
  }
  .commit-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 6px;
  }
  .amend-check {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
  }
  .commit-btn {
    padding: 5px 16px;
    border: none;
    border-radius: 6px;
    background: var(--accent);
    color: var(--bg-primary);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }
  .commit-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .commit-btn:hover:not(:disabled) {
    filter: brightness(1.1);
  }
  .no-status {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }
</style>
