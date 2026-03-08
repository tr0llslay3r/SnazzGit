<script lang="ts">
  import { commits, graphRows, repoInfo, refreshAll } from '$lib/stores/repo';
  import { selectedCommit, showBranchDialog, addToast } from '$lib/stores/ui';
  import { showContextMenu, type ContextMenuEntry } from '$lib/stores/contextmenu';
  import CommitGraph from './CommitGraph.svelte';
  import * as tauri from '$lib/utils/tauri';
  import type { CommitInfo, RefInfo } from '$lib/types';

  let container: HTMLDivElement;
  let scrollTop = $state(0);
  let clientHeight = $state(600);
  const ROW_HEIGHT = 28;
  const BUFFER = 10;

  let visibleStart = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER));
  let visibleCount = $derived(Math.ceil(clientHeight / ROW_HEIGHT) + BUFFER * 2);
  let visibleCommits = $derived($commits.slice(visibleStart, visibleStart + visibleCount));
  let totalHeight = $derived($commits.length * ROW_HEIGHT);

  function onScroll() {
    scrollTop = container.scrollTop;
    clientHeight = container.clientHeight;
  }

  function selectCommit(commit: CommitInfo) {
    if ($selectedCommit?.id === commit.id) {
      $selectedCommit = null;
    } else {
      $selectedCommit = commit;
    }
  }

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

  function onCommitContext(e: MouseEvent, commit: CommitInfo) {
    e.preventDefault();
    e.stopPropagation();
    const items: ContextMenuEntry[] = [
      { label: 'Copy Commit Hash', action: () => navigator.clipboard.writeText(commit.id) },
      { label: 'Copy Short Hash', action: () => navigator.clipboard.writeText(commit.short_id) },
      { label: 'Copy Message', action: () => navigator.clipboard.writeText(commit.summary) },
      { separator: true },
      { label: 'Create Branch Here', action: () => { $selectedCommit = commit; $showBranchDialog = true; } },
      { label: 'Checkout (Detached)', action: async () => {
        if (!$repoInfo) return;
        try {
          await tauri.checkoutBranch($repoInfo.path, commit.id);
          await refreshAll();
          addToast(`Checked out ${commit.short_id}`, 'success');
        } catch (err) { addToast(`Checkout failed: ${err}`, 'error'); }
      }},
      { label: 'Reset current branch to here', children: [
        { label: 'Soft (keep staged)', action: async () => {
          if (!$repoInfo) return;
          try {
            await tauri.resetToCommit($repoInfo.path, commit.id, 'soft');
            await refreshAll();
            addToast(`Soft reset to ${commit.short_id}`, 'success');
          } catch (err) { addToast(`Reset failed: ${err}`, 'error'); }
        }},
        { label: 'Mixed (keep unstaged)', action: async () => {
          if (!$repoInfo) return;
          try {
            await tauri.resetToCommit($repoInfo.path, commit.id, 'mixed');
            await refreshAll();
            addToast(`Mixed reset to ${commit.short_id}`, 'success');
          } catch (err) { addToast(`Reset failed: ${err}`, 'error'); }
        }},
        { label: 'Hard (discard all)', danger: true, action: async () => {
          if (!$repoInfo) return;
          try {
            await tauri.resetToCommit($repoInfo.path, commit.id, 'hard');
            await refreshAll();
            addToast(`Hard reset to ${commit.short_id}`, 'success');
          } catch (err) { addToast(`Reset failed: ${err}`, 'error'); }
        }},
      ]},
    ];
    showContextMenu(e.clientX, e.clientY, items);
  }

  function refBadgeClass(ref: RefInfo): string {
    switch (ref.ref_type) {
      case 'Head': return 'ref-head';
      case 'LocalBranch': return 'ref-local';
      case 'RemoteBranch': return 'ref-remote';
      case 'Tag': return 'ref-tag';
      default: return '';
    }
  }
</script>

<div class="commit-list-wrapper" bind:this={container} onscroll={onScroll}>
  <div class="virtual-scroll" style="height: {totalHeight}px">
    <div class="scroll-row">
      {#if $graphRows.length > 0}
        <CommitGraph {scrollTop} {clientHeight} totalRows={$commits.length} />
      {/if}
      <div class="rows-column">
        <div class="visible-rows" style="transform: translateY({visibleStart * ROW_HEIGHT}px)">
          {#each visibleCommits as commit (commit.id)}
            <button
              class="commit-row"
              class:selected={$selectedCommit?.id === commit.id}
              onclick={() => selectCommit(commit)}
              oncontextmenu={(e) => onCommitContext(e, commit)}
              style="height: {ROW_HEIGHT}px"
            >
              <span class="commit-hash">{commit.short_id}</span>
              <span class="commit-refs">
                {#each commit.refs as ref}
                  <span class="ref-badge {refBadgeClass(ref)}">{ref.name}</span>
                {/each}
              </span>
              <span class="commit-message">{commit.summary}</span>
              <span class="commit-author">{commit.author_name}</span>
              <span class="commit-date">{formatDate(commit.author_time)}</span>
            </button>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .commit-list-wrapper {
    overflow-y: auto;
    overflow-x: hidden;
    flex: 1;
  }
  .virtual-scroll {
    position: relative;
  }
  .scroll-row {
    display: flex;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }
  .rows-column {
    flex: 1;
    min-width: 0;
  }
  .visible-rows {
    position: relative;
  }
  .commit-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
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
  .commit-row:hover {
    background: var(--bg-surface);
  }
  .commit-row.selected {
    background: color-mix(in srgb, var(--accent) 20%, transparent);
    border-bottom-color: color-mix(in srgb, var(--accent) 30%, transparent);
  }
  .commit-hash {
    font-family: monospace;
    color: var(--accent);
    width: 70px;
    flex-shrink: 0;
  }
  .commit-refs {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }
  .ref-badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    font-weight: 600;
  }
  .ref-head {
    background: var(--accent);
    color: var(--bg-primary);
  }
  .ref-local {
    background: color-mix(in srgb, var(--accent) 30%, transparent);
    color: var(--accent);
  }
  .ref-remote {
    background: color-mix(in srgb, var(--accent-secondary) 30%, transparent);
    color: var(--accent-secondary);
  }
  .ref-tag {
    background: color-mix(in srgb, var(--warning) 30%, transparent);
    color: var(--warning);
  }
  .commit-message {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .commit-author {
    color: var(--text-secondary);
    width: 120px;
    flex-shrink: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .commit-date {
    color: var(--text-secondary);
    width: 70px;
    flex-shrink: 0;
    text-align: right;
  }
</style>
