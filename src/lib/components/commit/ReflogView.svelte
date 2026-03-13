<script lang="ts">
  import { repoInfo, refreshAll } from '$lib/stores/repo';
  import { showReflog, jumpToCommitId, addToast } from '$lib/stores/ui';
  import { showContextMenu, type ContextMenuEntry } from '$lib/stores/contextmenu';
  import * as tauri from '$lib/utils/tauri';
  import type { ReflogEntry } from '$lib/types';

  let entries = $state<ReflogEntry[]>([]);
  let loading = $state(false);

  $effect(() => {
    if ($showReflog && $repoInfo) {
      loading = true;
      tauri.getReflog($repoInfo.path, 500)
        .then((e) => { entries = e; })
        .catch((e) => { addToast(`Reflog failed: ${e}`, 'error'); entries = []; })
        .finally(() => { loading = false; });
    }
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
    $showReflog = false;
  }

  function onEntryContext(e: MouseEvent, entry: ReflogEntry) {
    e.preventDefault();
    const items: ContextMenuEntry[] = [
      { label: 'Jump to Commit', action: () => { $showReflog = false; $jumpToCommitId = entry.id; } },
      { label: 'Cherry-pick', action: async () => {
        if (!$repoInfo) return;
        try {
          const result = await tauri.cherryPickCommit($repoInfo.path, entry.id);
          await refreshAll();
          $showReflog = false;
          if (result.includes('conflicts')) {
            addToast(result, 'warning');
          } else {
            addToast(`Cherry-picked ${entry.short_id}`, 'success');
          }
        } catch (err) { addToast(`Cherry-pick failed: ${err}`, 'error'); }
      }},
      { separator: true },
      { label: 'Copy Hash', action: () => navigator.clipboard.writeText(entry.id) },
    ];
    showContextMenu(e.clientX, e.clientY, items);
  }
</script>

<div class="reflog-view">
  <div class="reflog-header">
    <span class="reflog-title">Reflog (HEAD)</span>
    <button class="close-btn" onclick={close}>&times;</button>
  </div>
  {#if loading}
    <div class="loading">Loading...</div>
  {:else}
    <div class="reflog-list">
      {#each entries as entry, i}
        <button
          class="reflog-row"
          onclick={() => { $showReflog = false; $jumpToCommitId = entry.id; }}
          oncontextmenu={(e) => onEntryContext(e, entry)}
        >
          <span class="idx">{i}</span>
          <span class="hash">{entry.short_id}</span>
          <span class="action-badge">{entry.action}</span>
          <span class="message">{entry.message}</span>
          <span class="date">{formatDate(entry.time)}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .reflog-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .reflog-header {
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
  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 16px;
    padding: 0 4px;
    line-height: 1;
  }
  .close-btn:hover { color: var(--text-primary); }
  .reflog-list {
    overflow-y: auto;
    flex: 1;
  }
  .reflog-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 12px;
    border: none;
    background: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 12px;
    width: 100%;
    text-align: left;
    white-space: nowrap;
  }
  .reflog-row:hover { background: var(--bg-surface); }
  .idx {
    color: var(--text-secondary);
    width: 30px;
    text-align: right;
    flex-shrink: 0;
    opacity: 0.6;
  }
  .hash {
    font-family: monospace;
    color: var(--accent);
    width: 70px;
    flex-shrink: 0;
  }
  .action-badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--accent) 20%, transparent);
    color: var(--accent);
    flex-shrink: 0;
    min-width: 60px;
    text-align: center;
  }
  .message {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .date {
    color: var(--text-secondary);
    width: 60px;
    flex-shrink: 0;
    text-align: right;
  }
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    color: var(--text-secondary);
  }
</style>
