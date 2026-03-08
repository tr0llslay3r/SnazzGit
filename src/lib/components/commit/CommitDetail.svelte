<script lang="ts">
  import DiffLineComponent from '$lib/components/diff/DiffLine.svelte';
  import HunkHeader from '$lib/components/diff/HunkHeader.svelte';
  import { selectedCommit, diffMode } from '$lib/stores/ui';
  import { repoInfo } from '$lib/stores/repo';
  import { showContextMenu } from '$lib/stores/contextmenu';
  import * as tauri from '$lib/utils/tauri';
  import type { DiffFile } from '$lib/types';

  let diffFiles = $state<DiffFile[]>([]);
  let expandedFile = $state<string | null>(null);

  $effect(() => {
    if ($selectedCommit && $repoInfo) {
      loadDiff();
    } else {
      diffFiles = [];
      expandedFile = null;
    }
  });

  async function loadDiff() {
    if (!$selectedCommit || !$repoInfo) return;
    try {
      diffFiles = await tauri.getCommitDiff($repoInfo.path, $selectedCommit.id);
    } catch {
      diffFiles = [];
    }
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  function toggleFile(path: string) {
    expandedFile = expandedFile === path ? null : path;
  }

  function fileStats(file: DiffFile): { added: number; removed: number } {
    let added = 0, removed = 0;
    for (const hunk of file.hunks) {
      for (const line of hunk.lines) {
        if (line.line_type === 'Addition') added++;
        else if (line.line_type === 'Deletion') removed++;
      }
    }
    return { added, removed };
  }
</script>

{#if $selectedCommit}
  <div class="commit-detail">
    <div class="detail-header">
      <div class="detail-row">
        <span class="detail-label">Commit</span>
        <code class="detail-value hash">{$selectedCommit.id}</code>
      </div>
      <div class="detail-row">
        <span class="detail-label">Author</span>
        <span class="detail-value">{$selectedCommit.author_name} &lt;{$selectedCommit.author_email}&gt;</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">Date</span>
        <span class="detail-value">{formatDate($selectedCommit.author_time)}</span>
      </div>
      {#if $selectedCommit.parent_ids.length > 0}
        <div class="detail-row">
          <span class="detail-label">Parents</span>
          <span class="detail-value">
            {#each $selectedCommit.parent_ids as parent, i}
              {#if i > 0}, {/if}
              <code class="parent-hash">{parent.slice(0, 8)}</code>
            {/each}
          </span>
        </div>
      {/if}
    </div>
    <div class="detail-message">
      <pre>{$selectedCommit.message}</pre>
    </div>
    {#if diffFiles.length > 0}
      <div class="detail-files">
        <div class="files-header">{diffFiles.length} file{diffFiles.length === 1 ? '' : 's'} changed</div>
        {#each diffFiles as file (file.path)}
          {@const stats = fileStats(file)}
          <div class="file-block" class:expanded={expandedFile === file.path}>
            <button class="file-entry" onclick={() => toggleFile(file.path)} oncontextmenu={(e) => { e.preventDefault(); e.stopPropagation(); showContextMenu(e.clientX, e.clientY, [{ label: 'Copy File Path', action: () => navigator.clipboard.writeText(file.path) }]); }}>
              <span class="file-expand">{expandedFile === file.path ? '\u25BC' : '\u25B6'}</span>
              <span class="file-name">{file.path}</span>
              <span class="file-stats">
                {#if stats.added > 0}<span class="stat-added">+{stats.added}</span>{/if}
                {#if stats.removed > 0}<span class="stat-removed">-{stats.removed}</span>{/if}
              </span>
            </button>
            {#if expandedFile === file.path}
              <div class="file-diff">
                {#each file.hunks as hunk}
                  <HunkHeader {hunk} showActions={false} />
                  {#each hunk.lines as line}
                    <DiffLineComponent {line} />
                  {/each}
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
{:else}
  <div class="no-selection">
    <p>Select a commit to view details</p>
  </div>
{/if}

<style>
  .commit-detail {
    padding: 12px;
    overflow-y: auto;
    height: 100%;
    font-size: 13px;
    box-sizing: border-box;
  }
  .detail-header {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }
  .detail-row {
    display: flex;
    gap: 12px;
    align-items: baseline;
  }
  .detail-label {
    color: var(--text-secondary);
    width: 60px;
    flex-shrink: 0;
    font-size: 12px;
  }
  .detail-value {
    color: var(--text-primary);
  }
  .hash {
    color: var(--accent);
    font-size: 12px;
    user-select: all;
  }
  .parent-hash {
    color: var(--accent-secondary);
    font-size: 12px;
  }
  .detail-message {
    padding: 10px 0;
    border-bottom: 1px solid var(--border);
  }
  .detail-message pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: inherit;
    font-size: 13px;
    color: var(--text-primary);
  }
  .detail-files {
    padding-top: 8px;
  }
  .files-header {
    font-size: 12px;
    color: var(--text-secondary);
    padding-bottom: 6px;
  }
  .file-block {
    border-radius: 4px;
    margin-bottom: 2px;
  }
  .file-entry {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    width: 100%;
    border: none;
    background: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
  }
  .file-entry:hover {
    background: var(--bg-surface);
  }
  .file-block.expanded .file-entry {
    background: var(--bg-surface);
  }
  .file-expand {
    font-size: 9px;
    color: var(--text-secondary);
    width: 12px;
    flex-shrink: 0;
  }
  .file-name {
    font-family: monospace;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-stats {
    display: flex;
    gap: 6px;
    font-family: monospace;
    font-size: 11px;
    flex-shrink: 0;
  }
  .stat-added { color: var(--success); }
  .stat-removed { color: var(--danger); }
  .file-diff {
    border: 1px solid var(--border);
    border-radius: 4px;
    margin: 4px 0 8px 18px;
    overflow-x: auto;
  }
  .no-selection {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
    font-size: 14px;
  }
</style>
