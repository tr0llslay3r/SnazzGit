<script lang="ts">
  import { repoInfo } from '$lib/stores/repo';
  import { fileHistoryPath, selectedCommit, addToast } from '$lib/stores/ui';
  import * as tauri from '$lib/utils/tauri';
  import type { CommitInfo } from '$lib/types';

  let historyCommits = $state<CommitInfo[]>([]);
  let loading = $state(false);

  $effect(() => {
    const fp = $fileHistoryPath;
    if (!fp || !$repoInfo) {
      historyCommits = [];
      return;
    }
    loading = true;
    tauri.fileHistory($repoInfo.path, fp, 200)
      .then((commits) => { historyCommits = commits; })
      .catch((e) => { addToast(`File history failed: ${e}`, 'error'); historyCommits = []; })
      .finally(() => { loading = false; });
  });

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const now = Date.now();
    const diff = now - date.getTime();
    if (diff < 60000) return 'just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    if (diff < 2592000000) return `${Math.floor(diff / 86400000)}d ago`;
    return date.toLocaleDateString();
  }

  function close() {
    $fileHistoryPath = null;
  }
</script>

<div class="file-history">
  <div class="file-history-header">
    <span class="file-history-title">History: {$fileHistoryPath}</span>
    <button class="close-btn" onclick={close} title="Close file history">&times;</button>
  </div>
  {#if loading}
    <div class="loading">Loading...</div>
  {:else if historyCommits.length === 0}
    <div class="empty">No history found</div>
  {:else}
    <div class="history-list">
      {#each historyCommits as commit (commit.id)}
        <button
          class="history-row"
          class:selected={$selectedCommit?.id === commit.id}
          onclick={() => { $selectedCommit = commit; }}
        >
          <span class="hash">{commit.short_id}</span>
          <span class="message">{commit.summary}</span>
          <span class="author">{commit.author_name}</span>
          <span class="date">{formatDate(commit.author_time)}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .file-history {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .file-history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .file-history-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 16px;
    padding: 0 4px;
    line-height: 1;
  }
  .close-btn:hover {
    color: var(--text-primary);
  }
  .history-list {
    overflow-y: auto;
    flex: 1;
  }
  .history-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    border: none;
    background: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 12px;
    width: 100%;
    text-align: left;
    white-space: nowrap;
    border-bottom: 1px solid transparent;
  }
  .history-row:hover {
    background: var(--bg-surface);
  }
  .history-row.selected {
    background: color-mix(in srgb, var(--accent) 20%, transparent);
  }
  .hash {
    font-family: monospace;
    color: var(--accent);
    width: 70px;
    flex-shrink: 0;
  }
  .message {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .author {
    color: var(--text-secondary);
    width: 100px;
    flex-shrink: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .date {
    color: var(--text-secondary);
    width: 60px;
    flex-shrink: 0;
    text-align: right;
  }
  .loading, .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    color: var(--text-secondary);
    font-size: 13px;
  }
</style>
