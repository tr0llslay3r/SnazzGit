<script lang="ts">
  import DiffLineComponent from './DiffLine.svelte';
  import HunkHeader from './HunkHeader.svelte';
  import ConflictView from './ConflictView.svelte';
  import { diffMode, selectedFile, selectedFileStaged } from '$lib/stores/ui';
  import { repoInfo, workingStatus } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';
  import type { DiffFile } from '$lib/types';

  interface Props {
    file?: DiffFile | null;
    showHunkActions?: boolean;
  }

  let { file = null, showHunkActions = false }: Props = $props();

  let activeDiff = $state<DiffFile | null>(null);
  let loadError = $state<string | null>(null);

  // Plain counter to cancel stale requests (not reactive)
  let requestId = 0;

  let isConflicted = $derived(
    !$selectedFileStaged &&
      ($workingStatus?.unstaged.some(
        (f) => f.path === $selectedFile && f.status === 'Conflicted'
      ) ??
        false)
  );

  async function fetchDiff(repoPath: string, filePath: string, staged: boolean, id: number) {
    try {
      const result = await tauri.getWorkingDiff(repoPath, filePath, staged);
      if (id === requestId) {
        activeDiff = result;
        loadError = null;
      }
    } catch (err) {
      if (id === requestId) {
        activeDiff = null;
        loadError = String(err);
      }
    }
  }

  $effect(() => {
    if (file) {
      activeDiff = file;
      loadError = null;
      return;
    }

    if (isConflicted) {
      activeDiff = null;
      loadError = null;
      return;
    }

    const filePath = $selectedFile;
    const repo = $repoInfo;
    const staged = $selectedFileStaged;

    if (!filePath || !repo) {
      activeDiff = null;
      loadError = null;
      return;
    }

    const id = ++requestId;
    fetchDiff(repo.path, filePath, staged, id);
  });
</script>

<div class="diff-view">
  {#if isConflicted && $selectedFile}
    <ConflictView filePath={$selectedFile} />
  {:else if activeDiff && activeDiff.hunks.length > 0}
    <div class="diff-toolbar">
      <span class="diff-path">{activeDiff.path}</span>
      <div class="diff-mode-toggle">
        <button
          class="mode-btn"
          class:active={$diffMode === 'unified'}
          onclick={() => $diffMode = 'unified'}
        >Unified</button>
        <button
          class="mode-btn"
          class:active={$diffMode === 'split'}
          onclick={() => $diffMode = 'split'}
        >Split</button>
      </div>
    </div>
    <div class="diff-content">
      {#each activeDiff.hunks as hunk}
        <HunkHeader {hunk} showActions={showHunkActions} />
        {#each hunk.lines as line}
          <DiffLineComponent {line} />
        {/each}
      {/each}
    </div>
  {:else if loadError}
    <div class="no-diff">
      <p>{loadError}</p>
    </div>
  {:else if $selectedFile}
    <div class="no-diff">
      <p>No changes to display</p>
    </div>
  {:else}
    <div class="no-diff">
      <p>Select a file to view diff</p>
    </div>
  {/if}
</div>

<style>
  .diff-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .diff-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .diff-path {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-primary);
  }
  .diff-mode-toggle {
    display: flex;
    border: 1px solid var(--border);
    border-radius: 4px;
    overflow: hidden;
  }
  .mode-btn {
    padding: 2px 10px;
    border: none;
    background: none;
    color: var(--text-secondary);
    font-size: 11px;
    cursor: pointer;
  }
  .mode-btn.active {
    background: var(--accent);
    color: var(--bg-primary);
  }
  .diff-content {
    overflow: auto;
    flex: 1;
  }
  .no-diff {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }
</style>
