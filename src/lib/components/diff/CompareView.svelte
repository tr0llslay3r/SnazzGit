<script lang="ts">
  import { repoInfo } from '$lib/stores/repo';
  import { compareRefs, selectedCommit, addToast } from '$lib/stores/ui';
  import * as tauri from '$lib/utils/tauri';
  import type { DiffFile } from '$lib/types';
  import DiffLineComponent from './DiffLine.svelte';
  import HunkHeader from './HunkHeader.svelte';
  import { computeWordDiff } from '$lib/utils/worddiff';
  import type { WordDiffSegment } from '$lib/utils/worddiff';

  let diffFiles = $state<DiffFile[]>([]);
  let loading = $state(false);
  let selectedFilePath = $state<string | null>(null);

  $effect(() => {
    const refs = $compareRefs;
    if (!refs || !$repoInfo) {
      diffFiles = [];
      return;
    }
    loading = true;
    tauri.diffRefs($repoInfo.path, refs.from, refs.to)
      .then((files) => { diffFiles = files; })
      .catch((e) => { addToast(`Compare failed: ${e}`, 'error'); diffFiles = []; })
      .finally(() => { loading = false; });
  });

  let selectedDiff = $derived(
    selectedFilePath ? diffFiles.find(f => f.path === selectedFilePath) ?? null : null
  );

  function close() {
    $compareRefs = null;
    selectedFilePath = null;
  }

  function getWordSegments(hunk: import('$lib/types').DiffHunk, lineIndex: number): WordDiffSegment[] | undefined {
    const line = hunk.lines[lineIndex];
    if (line.line_type === 'Deletion') {
      const next = hunk.lines[lineIndex + 1];
      if (next && next.line_type === 'Addition') {
        return computeWordDiff(line.content, next.content).oldSegments;
      }
    } else if (line.line_type === 'Addition') {
      const prev = hunk.lines[lineIndex - 1];
      if (prev && prev.line_type === 'Deletion') {
        return computeWordDiff(prev.content, line.content).newSegments;
      }
    }
    return undefined;
  }
</script>

<div class="compare-view">
  <div class="compare-header">
    <span class="compare-title">
      Comparing: <strong>{$compareRefs?.from}</strong> ... <strong>{$compareRefs?.to}</strong>
      ({diffFiles.length} file{diffFiles.length !== 1 ? 's' : ''} changed)
    </span>
    <button class="close-btn" onclick={close} title="Close compare">&times;</button>
  </div>
  <div class="compare-body">
    <div class="file-list">
      {#if loading}
        <div class="loading">Loading...</div>
      {:else}
        {#each diffFiles as file (file.path)}
          <button
            class="file-item"
            class:selected={selectedFilePath === file.path}
            onclick={() => { selectedFilePath = file.path; }}
          >
            {file.path}
          </button>
        {/each}
      {/if}
    </div>
    <div class="diff-content">
      {#if selectedDiff}
        {#each selectedDiff.hunks as hunk}
          <HunkHeader {hunk} />
          {#each hunk.lines as line, i}
            <DiffLineComponent {line} wordSegments={getWordSegments(hunk, i)} />
          {/each}
        {/each}
      {:else if !loading}
        <div class="empty">Select a file to view diff</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .compare-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .compare-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--text-primary);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .compare-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
  .compare-body {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  .file-list {
    width: 220px;
    flex-shrink: 0;
    overflow-y: auto;
    border-right: 1px solid var(--border);
    background: var(--bg-secondary);
  }
  .file-item {
    display: block;
    width: 100%;
    padding: 4px 10px;
    border: none;
    background: none;
    color: var(--text-primary);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .file-item:hover {
    background: var(--bg-surface);
  }
  .file-item.selected {
    background: color-mix(in srgb, var(--accent) 20%, transparent);
  }
  .diff-content {
    flex: 1;
    overflow: auto;
    background: var(--bg-primary);
  }
  .loading, .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    color: var(--text-secondary);
    font-size: 13px;
    height: 100%;
  }
</style>
