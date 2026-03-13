<script lang="ts">
  import DiffLineComponent from './DiffLine.svelte';
  import HunkHeader from './HunkHeader.svelte';
  import ConflictView from './ConflictView.svelte';
  import { diffMode, selectedFile, selectedFileStaged, addToast } from '$lib/stores/ui';
  import { repoInfo, workingStatus, refreshStatus } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';
  import { computeWordDiff, type WordDiffSegment } from '$lib/utils/worddiff';
  import type { DiffFile, DiffHunk, DiffLine } from '$lib/types';

  interface Props {
    file?: DiffFile | null;
    showHunkActions?: boolean;
  }

  let { file = null, showHunkActions = false }: Props = $props();

  let activeDiff = $state<DiffFile | null>(null);
  let loadError = $state<string | null>(null);
  let oldImageSrc = $state<string | null>(null);
  let newImageSrc = $state<string | null>(null);

  let requestId = 0;

  const IMAGE_EXTS = new Set(['.png', '.jpg', '.jpeg', '.gif', '.bmp', '.webp', '.svg', '.ico']);

  function isImageFile(path: string): boolean {
    const ext = path.substring(path.lastIndexOf('.')).toLowerCase();
    return IMAGE_EXTS.has(ext);
  }

  function mimeType(path: string): string {
    const ext = path.substring(path.lastIndexOf('.')).toLowerCase();
    const map: Record<string, string> = {
      '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg',
      '.gif': 'image/gif', '.bmp': 'image/bmp', '.webp': 'image/webp',
      '.svg': 'image/svg+xml', '.ico': 'image/x-icon',
    };
    return map[ext] ?? 'image/png';
  }

  let repoPath = $derived($repoInfo?.path ?? null);

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
      if (id !== requestId) return;
      activeDiff = result;
      loadError = null;
    } catch (err) {
      if (id === requestId) {
        activeDiff = null;
        loadError = String(err);
      }
    }
  }

  async function loadImageDiff(repoPath: string, filePath: string) {
    const mime = mimeType(filePath);
    try {
      // Old version from HEAD
      const oldData = await tauri.readFileAtRef(repoPath, filePath, 'HEAD');
      oldImageSrc = oldData ? `data:${mime};base64,${oldData}` : null;
    } catch {
      oldImageSrc = null;
    }
    try {
      // New version from working tree
      const newData = await tauri.readFileAtRef(repoPath, filePath);
      newImageSrc = newData ? `data:${mime};base64,${newData}` : null;
    } catch {
      newImageSrc = null;
    }
  }

  $effect(() => {
    if (file) {
      activeDiff = file;
      loadError = null;
      oldImageSrc = null;
      newImageSrc = null;
      return;
    }

    // Read all reactive deps before any early returns to ensure proper tracking.
    // Use repoPath (string) instead of $repoInfo (object) to avoid re-triggering
    // when the watcher refreshes repo info with a new object reference.
    const filePath = $selectedFile;
    const rp = repoPath;
    const staged = $selectedFileStaged;
    const conflicted = isConflicted;

    if (conflicted) {
      activeDiff = null;
      loadError = null;
      return;
    }

    if (!filePath || !rp) {
      activeDiff = null;
      loadError = null;
      oldImageSrc = null;
      newImageSrc = null;
      return;
    }

    const id = ++requestId;
    if (isImageFile(filePath)) {
      activeDiff = null;
      loadError = null;
      loadImageDiff(rp, filePath);
    } else {
      oldImageSrc = null;
      newImageSrc = null;
      fetchDiff(rp, filePath, staged, id);
    }
  });

  function hunkLines(hunk: DiffHunk): string[] {
    return hunk.lines.map((l) => {
      const prefix =
        l.line_type === 'Addition' ? '+' : l.line_type === 'Deletion' ? '-' : ' ';
      return prefix + l.content;
    });
  }

  async function onStageHunk(hunk: DiffHunk) {
    if (!$repoInfo || !$selectedFile) return;
    try {
      await tauri.stageHunk(
        $repoInfo.path,
        $selectedFile,
        hunk.old_start,
        hunk.old_lines,
        hunk.new_start,
        hunk.new_lines,
        hunk.header,
        hunkLines(hunk),
      );
      await refreshStatus();
      // Re-fetch diff
      const id = ++requestId;
      fetchDiff($repoInfo.path, $selectedFile, $selectedFileStaged, id);
    } catch (err) {
      addToast(`Stage hunk failed: ${err}`, 'error');
    }
  }

  // Compute word-level diffs for adjacent deletion/addition pairs
  function getWordSegments(hunk: DiffHunk): Map<number, WordDiffSegment[]> {
    const map = new Map<number, WordDiffSegment[]>();
    const lines = hunk.lines;
    let i = 0;
    while (i < lines.length) {
      // Find contiguous deletion block
      const delStart = i;
      while (i < lines.length && lines[i].line_type === 'Deletion') i++;
      const delEnd = i;
      // Find contiguous addition block
      const addStart = i;
      while (i < lines.length && lines[i].line_type === 'Addition') i++;
      const addEnd = i;

      const delCount = delEnd - delStart;
      const addCount = addEnd - addStart;

      // Only compute word diff for 1:1 pairs (most common case)
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

      if (i === delStart) i++; // skip context lines
    }
    return map;
  }

  async function onUnstageHunk(hunk: DiffHunk) {
    if (!$repoInfo || !$selectedFile) return;
    try {
      await tauri.unstageHunk(
        $repoInfo.path,
        $selectedFile,
        hunk.old_start,
        hunk.old_lines,
        hunk.new_start,
        hunk.new_lines,
        hunk.header,
        hunkLines(hunk),
      );
      await refreshStatus();
      const id = ++requestId;
      fetchDiff($repoInfo.path, $selectedFile, $selectedFileStaged, id);
    } catch (err) {
      addToast(`Unstage hunk failed: ${err}`, 'error');
    }
  }
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
        {@const wordSegments = getWordSegments(hunk)}
        <HunkHeader
          {hunk}
          showActions={showHunkActions}
          onStage={$selectedFileStaged ? undefined : () => onStageHunk(hunk)}
          onDiscard={$selectedFileStaged ? () => onUnstageHunk(hunk) : undefined}
        />
        {#each hunk.lines as line, idx}
          <DiffLineComponent {line} wordSegments={wordSegments.get(idx)} />
        {/each}
      {/each}
    </div>
  {:else if oldImageSrc || newImageSrc}
    <div class="diff-toolbar">
      <span class="diff-path">{$selectedFile}</span>
      <span class="diff-path" style="opacity: 0.6">Image diff</span>
    </div>
    <div class="image-diff">
      {#if oldImageSrc}
        <div class="image-panel">
          <div class="image-label">Before</div>
          <img src={oldImageSrc} alt="Old version" />
        </div>
      {/if}
      {#if newImageSrc}
        <div class="image-panel">
          <div class="image-label">After</div>
          <img src={newImageSrc} alt="New version" />
        </div>
      {/if}
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
  .image-diff {
    display: flex;
    gap: 16px;
    padding: 16px;
    overflow: auto;
    flex: 1;
    align-items: flex-start;
    justify-content: center;
  }
  .image-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px;
    background: var(--bg-secondary);
    max-width: 50%;
  }
  .image-panel img {
    max-width: 100%;
    max-height: 400px;
    object-fit: contain;
    border-radius: 4px;
    background: repeating-conic-gradient(#80808020 0% 25%, transparent 0% 50%) 50% / 16px 16px;
  }
  .image-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-secondary);
  }
  .no-diff {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }
</style>
