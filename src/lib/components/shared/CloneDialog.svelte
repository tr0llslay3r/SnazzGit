<script lang="ts">
  import { showCloneDialog, showCredentialDialog, pendingCredentialRequest, addToast } from '$lib/stores/ui';
  import { repoInfo, refreshAll, loadRecentRepos } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';
  import { open } from '@tauri-apps/plugin-dialog';
  import { listen } from '@tauri-apps/api/event';
  import { onDestroy } from 'svelte';

  let url = $state('');
  let directory = $state('');
  let isCloning = $state(false);
  let progress = $state({ received: 0, total: 0, bytes: 0 });
  let inputEl: HTMLInputElement = $state(null!);

  $effect(() => {
    if ($showCloneDialog && inputEl) {
      inputEl.focus();
    }
  });

  let repoName = $derived(() => {
    if (!url) return '';
    const parts = url.replace(/\.git\/?$/, '').split('/');
    return parts[parts.length - 1] || '';
  });

  let clonePath = $derived(() => {
    if (!directory || !repoName()) return '';
    return `${directory}/${repoName()}`;
  });

  let canClone = $derived(() => {
    return url.trim() !== '' && directory !== '' && !isCloning;
  });

  let progressPercent = $derived(() => {
    if (progress.total === 0) return 0;
    return Math.round((progress.received / progress.total) * 100);
  });

  let unlistenProgress: (() => void) | null = null;

  async function setupProgressListener() {
    unlistenProgress = await listen<{ received_objects: number; total_objects: number; received_bytes: number }>('git-progress', (event) => {
      progress = {
        received: event.payload.received_objects,
        total: event.payload.total_objects,
        bytes: event.payload.received_bytes,
      };
    });
  }

  onDestroy(() => {
    unlistenProgress?.();
  });

  function isAuthError(e: unknown): boolean {
    return String(e).includes('Authentication failed');
  }

  async function browseDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      directory = selected as string;
    }
  }

  async function doClone() {
    if (!canClone()) return;
    isCloning = true;
    progress = { received: 0, total: 0, bytes: 0 };

    await setupProgressListener();

    try {
      const workdir = await tauri.cloneRepo(url.trim(), clonePath());
      const info = await tauri.openRepository(workdir);
      $repoInfo = info;
      await tauri.addRecentRepo(info.path, info.name);
      await loadRecentRepos();
      await refreshAll();
      addToast(`Cloned ${info.name}`, 'success');
      close();
    } catch (e) {
      if (isAuthError(e)) {
        $pendingCredentialRequest = {
          operation: 'clone',
          url: url.trim(),
          clonePath: clonePath(),
        };
        $showCredentialDialog = true;
        // Keep clone dialog open but stop loading state
      } else {
        addToast(`Clone failed: ${e}`, 'error');
      }
    } finally {
      isCloning = false;
      unlistenProgress?.();
      unlistenProgress = null;
    }
  }

  function close() {
    url = '';
    directory = '';
    isCloning = false;
    progress = { received: 0, total: 0, bytes: 0 };
    $showCloneDialog = false;
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !isCloning) doClone();
    if (e.key === 'Escape' && !isCloning) close();
  }
</script>

{#if $showCloneDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="dialog-overlay" onclick={() => { if (!isCloning) close(); }}>
    <!-- svelte-ignore a11y_interactive_supports_focus, a11y_no_static_element_interactions -->
    <div class="dialog" onclick={(e) => e.stopPropagation()} onkeydown={onKeydown} role="dialog" tabindex="-1">
      <h3 class="dialog-title">Clone Repository</h3>
      <div class="dialog-fields">
        <input
          class="dialog-input"
          type="text"
          placeholder="https://github.com/user/repo.git"
          bind:this={inputEl}
          bind:value={url}
          disabled={isCloning}
        />
        <div class="directory-row">
          <div class="directory-display" class:empty={!directory}>
            {directory || 'Choose directory...'}
          </div>
          <button class="btn-secondary" onclick={browseDirectory} disabled={isCloning}>Browse</button>
        </div>
        {#if clonePath()}
          <p class="clone-path-preview">Will clone to: <strong>{clonePath()}</strong></p>
        {/if}
      </div>

      {#if isCloning}
        <div class="progress-section">
          <div class="progress-bar-track">
            <div class="progress-bar-fill" style="width: {progressPercent()}%"></div>
          </div>
          <div class="progress-info">
            <span>{progressPercent()}% ({progress.received}/{progress.total} objects)</span>
            <span>{formatBytes(progress.bytes)}</span>
          </div>
        </div>
      {/if}

      <div class="dialog-actions">
        <button class="btn-secondary" onclick={close} disabled={isCloning}>Cancel</button>
        <button class="btn-primary" onclick={doClone} disabled={!canClone()}>
          {isCloning ? 'Cloning...' : 'Clone'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    width: 500px;
    max-width: 90vw;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }
  .dialog-title {
    margin: 0 0 16px;
    font-size: 16px;
    color: var(--text-primary);
  }
  .dialog-fields {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .dialog-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 14px;
    outline: none;
    box-sizing: border-box;
  }
  .dialog-input:focus {
    border-color: var(--accent);
  }
  .dialog-input:disabled {
    opacity: 0.6;
  }
  .directory-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .directory-display {
    flex: 1;
    padding: 8px 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .directory-display.empty {
    color: var(--text-secondary);
  }
  .clone-path-preview {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
    word-break: break-all;
  }
  .clone-path-preview strong {
    color: var(--text-primary);
  }
  .progress-section {
    margin-top: 14px;
  }
  .progress-bar-track {
    height: 6px;
    background: var(--bg-primary);
    border-radius: 3px;
    overflow: hidden;
  }
  .progress-bar-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.2s;
  }
  .progress-info {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--text-secondary);
    margin-top: 4px;
  }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
  .btn-secondary {
    padding: 6px 16px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 13px;
  }
  .btn-primary {
    padding: 6px 16px;
    border: none;
    border-radius: 6px;
    background: var(--accent);
    color: var(--bg-primary);
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
  }
  .btn-primary:disabled {
    opacity: 0.4;
  }
  .btn-secondary:disabled {
    opacity: 0.4;
  }
</style>
