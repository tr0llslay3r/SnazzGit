<script lang="ts">
  import DiffLineComponent from '$lib/components/diff/DiffLine.svelte';
  import HunkHeader from '$lib/components/diff/HunkHeader.svelte';
  import { repoInfo } from '$lib/stores/repo';
  import { selectedStash } from '$lib/stores/ui';
  import { computeWordDiff, type WordDiffSegment } from '$lib/utils/worddiff';
  import * as tauri from '$lib/utils/tauri';
  import type { DiffFile, DiffHunk } from '$lib/types';

  let files = $state<DiffFile[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  let repoPath = $derived($repoInfo?.path ?? null);

  $effect(() => {
    const stash = $selectedStash;
    const rp = repoPath;
    if (!stash || !rp) {
      files = [];
      return;
    }
    loading = true;
    error = null;
    tauri.stashDiff(rp, stash.commit_id)
      .then((result) => { files = result; })
      .catch((e) => { error = String(e); files = []; })
      .finally(() => { loading = false; });
  });

  function getWordSegments(hunk: DiffHunk): Map<number, WordDiffSegment[]> {
    const map = new Map<number, WordDiffSegment[]>();
    const lines = hunk.lines;
    let i = 0;
    while (i < lines.length) {
      const delStart = i;
      while (i < lines.length && lines[i].line_type === 'Deletion') i++;
      const delEnd = i;
      const addStart = i;
      while (i < lines.length && lines[i].line_type === 'Addition') i++;
      const addEnd = i;
      const delCount = delEnd - delStart;
      const addCount = addEnd - addStart;
      if (delCount > 0 && addCount > 0) {
        const pairs = Math.min(delCount, addCount);
        for (let p = 0; p < pairs; p++) {
          const { oldSegments, newSegments } = computeWordDiff(
            lines[delStart + p].content,
            lines[addStart + p].content
          );
          map.set(delStart + p, oldSegments);
          map.set(addStart + p, newSegments);
        }
      }
      if (i === delStart) i++;
    }
    return map;
  }
</script>

<div class="stash-preview">
  <div class="stash-header">
    <span class="stash-title">Stash: {$selectedStash?.message || 'stash@{' + ($selectedStash?.index ?? 0) + '}'}</span>
    <button class="close-btn" onclick={() => $selectedStash = null}>&#x2715;</button>
  </div>
  {#if loading}
    <div class="stash-loading">Loading stash diff...</div>
  {:else if error}
    <div class="stash-error">{error}</div>
  {:else if files.length === 0}
    <div class="stash-empty">No changes in this stash</div>
  {:else}
    <div class="stash-files">
      {#each files as file}
        <div class="stash-file">
          <div class="file-header">{file.path}</div>
          {#each file.hunks as hunk}
            {@const wordSegments = getWordSegments(hunk)}
            <HunkHeader {hunk} />
            {#each hunk.lines as line, idx}
              <DiffLineComponent {line} wordSegments={wordSegments.get(idx)} />
            {/each}
          {/each}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .stash-preview {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .stash-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .stash-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 14px;
    padding: 2px 6px;
    border-radius: 4px;
  }
  .close-btn:hover {
    background: var(--bg-surface);
    color: var(--text-primary);
  }
  .stash-files {
    overflow: auto;
    flex: 1;
  }
  .file-header {
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
    font-family: monospace;
    color: var(--accent);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  .stash-loading, .stash-error, .stash-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--text-secondary);
    font-size: 13px;
  }
  .stash-error { color: var(--danger); }
</style>
