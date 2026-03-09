<script lang="ts">
  import { repoInfo, refreshStatus } from '$lib/stores/repo';
  import { addToast } from '$lib/stores/ui';
  import * as tauri from '$lib/utils/tauri';
  import type { ConflictFile, ConflictHunk } from '$lib/types';

  interface Props {
    filePath: string;
  }

  let { filePath }: Props = $props();

  let conflictFile = $state<ConflictFile | null>(null);
  let loadError = $state<string | null>(null);
  let isLoading = $state(false);
  // Map from conflict_index -> 'ours' | 'theirs'
  let choices = $state(new Map<number, 'ours' | 'theirs'>());
  let isSaving = $state(false);

  let conflictCount = $derived(
    conflictFile
      ? conflictFile.hunks.filter((h) => h.kind === 'ours').length
      : 0
  );
  let resolvedCount = $derived(choices.size);
  let allResolved = $derived(conflictCount > 0 && resolvedCount === conflictCount);
  let unresolvedCount = $derived(conflictCount - resolvedCount);

  $effect(() => {
    if (filePath && $repoInfo) {
      loadConflict();
    }
  });

  async function loadConflict() {
    isLoading = true;
    loadError = null;
    conflictFile = null;
    choices = new Map();
    try {
      conflictFile = await tauri.getConflictDiff($repoInfo!.path, filePath);
    } catch (err) {
      loadError = String(err);
    } finally {
      isLoading = false;
    }
  }

  function acceptOurs(idx: number) {
    const next = new Map(choices);
    next.set(idx, 'ours');
    choices = next;
  }

  function acceptTheirs(idx: number) {
    const next = new Map(choices);
    next.set(idx, 'theirs');
    choices = next;
  }

  async function acceptAllOurs() {
    if (!$repoInfo) return;
    try {
      await tauri.resolveConflictOursTheirs($repoInfo.path, filePath, true);
      await refreshStatus();
      addToast('Accepted all ours — file staged', 'success');
    } catch (e) {
      addToast(`Failed: ${e}`, 'error');
    }
  }

  async function acceptAllTheirs() {
    if (!$repoInfo) return;
    try {
      await tauri.resolveConflictOursTheirs($repoInfo.path, filePath, false);
      await refreshStatus();
      addToast('Accepted all theirs — file staged', 'success');
    } catch (e) {
      addToast(`Failed: ${e}`, 'error');
    }
  }

  function buildContent(): string {
    if (!conflictFile) return '';
    const parts: string[] = [];
    for (const hunk of conflictFile.hunks) {
      if (hunk.kind === 'context') {
        parts.push(...hunk.lines);
      } else if (hunk.kind === 'ours' && hunk.conflict_index !== null) {
        if (choices.get(hunk.conflict_index) === 'ours') {
          parts.push(...hunk.lines);
        }
      } else if (hunk.kind === 'theirs' && hunk.conflict_index !== null) {
        if (choices.get(hunk.conflict_index) === 'theirs') {
          parts.push(...hunk.lines);
        }
      }
    }
    return parts.join('');
  }

  async function saveAndStage() {
    if (!conflictFile || !$repoInfo || !allResolved) return;
    isSaving = true;
    try {
      const content = buildContent();
      await tauri.saveResolvedConflict($repoInfo.path, filePath, content);
      await refreshStatus();
      addToast('Conflict resolved and staged', 'success');
    } catch (e) {
      addToast(`Failed to save: ${e}`, 'error');
    } finally {
      isSaving = false;
    }
  }

  // Group hunks into display segments: pairs of [ours, theirs] with same conflict_index
  type ConflictPair = { idx: number; ours: ConflictHunk; theirs: ConflictHunk };
  type Segment =
    | { type: 'context'; hunk: ConflictHunk }
    | { type: 'conflict'; pair: ConflictPair };

  let segments = $derived((): Segment[] => {
    if (!conflictFile) return [];
    const result: Segment[] = [];
    const pairs = new Map<number, Partial<ConflictPair>>();

    for (const hunk of conflictFile.hunks) {
      if (hunk.kind === 'context') {
        result.push({ type: 'context', hunk });
      } else if (hunk.kind === 'ours' && hunk.conflict_index !== null) {
        const idx = hunk.conflict_index;
        pairs.set(idx, { ...(pairs.get(idx) ?? { idx }), ours: hunk });
      } else if (hunk.kind === 'theirs' && hunk.conflict_index !== null) {
        const idx = hunk.conflict_index;
        const pair = { ...(pairs.get(idx) ?? { idx }), theirs: hunk } as ConflictPair;
        pairs.set(idx, pair);
        if (pair.ours && pair.theirs) {
          result.push({ type: 'conflict', pair });
        }
      }
    }
    return result;
  });
</script>

<div class="conflict-view">
  <!-- Toolbar -->
  <div class="conflict-toolbar">
    <span class="conflict-path">{filePath}</span>
    <div class="toolbar-right">
      {#if isLoading}
        <span class="status-badge">Loading…</span>
      {:else if conflictCount > 0}
        {#if unresolvedCount > 0}
          <span class="badge badge-danger">{unresolvedCount} unresolved</span>
        {:else}
          <span class="badge badge-success">All resolved</span>
        {/if}
      {/if}
      <button class="action-btn" onclick={acceptAllOurs} disabled={isLoading || !conflictFile}>
        Accept All Ours
      </button>
      <button class="action-btn" onclick={acceptAllTheirs} disabled={isLoading || !conflictFile}>
        Accept All Theirs
      </button>
    </div>
  </div>

  <!-- Body -->
  <div class="conflict-body">
    {#if isLoading}
      <div class="message">Loading conflict…</div>
    {:else if loadError}
      <div class="message error">{loadError}</div>
    {:else if conflictFile}
      <div class="conflict-content">
        {#each segments() as seg}
          {#if seg.type === 'context'}
            {#each seg.hunk.lines as line, i}
              <div class="diff-line line-context">
                <span class="line-no">{seg.hunk.start_line + i}</span>
                <span class="line-prefix"> </span>
                <span class="line-content">{line}</span>
              </div>
            {/each}
          {:else if seg.type === 'conflict'}
            {@const pair = seg.pair}
            {@const choice = choices.get(pair.idx)}
            <!-- Ours header -->
            <div class="hunk-header hunk-header-ours">
              <span class="hunk-label">
                &lt;&lt;&lt;&lt;&lt;&lt;&lt; {conflictFile.our_label} (ours)
              </span>
              <div class="hunk-actions">
                {#if choice === 'ours'}
                  <span class="resolved-badge badge-success">Accepted</span>
                {:else}
                  <button class="accept-btn accept-ours" onclick={() => acceptOurs(pair.idx)}>
                    Accept Ours
                  </button>
                {/if}
              </div>
            </div>
            <!-- Ours lines -->
            {#each pair.ours.lines as line, i}
              <div class="diff-line line-ours" class:dimmed={choice === 'theirs'}>
                <span class="line-no">{pair.ours.start_line + i}</span>
                <span class="line-prefix">+</span>
                <span class="line-content">{line}</span>
              </div>
            {/each}
            <!-- Separator -->
            <div class="hunk-separator">=======</div>
            <!-- Theirs header -->
            <div class="hunk-header hunk-header-theirs">
              <span class="hunk-label">
                &gt;&gt;&gt;&gt;&gt;&gt;&gt; {conflictFile.their_label} (theirs)
              </span>
              <div class="hunk-actions">
                {#if choice === 'theirs'}
                  <span class="resolved-badge badge-theirs">Accepted</span>
                {:else}
                  <button class="accept-btn accept-theirs" onclick={() => acceptTheirs(pair.idx)}>
                    Accept Theirs
                  </button>
                {/if}
              </div>
            </div>
            <!-- Theirs lines -->
            {#each pair.theirs.lines as line, i}
              <div class="diff-line line-theirs" class:dimmed={choice === 'ours'}>
                <span class="line-no">{pair.theirs.start_line + i}</span>
                <span class="line-prefix">+</span>
                <span class="line-content">{line}</span>
              </div>
            {/each}
            <!-- End marker -->
            <div class="hunk-end">
              &gt;&gt;&gt;&gt;&gt;&gt;&gt; end of conflict {pair.idx + 1}
            </div>
          {/if}
        {/each}
      </div>

      <!-- Footer when all resolved -->
      {#if allResolved}
        <div class="conflict-footer">
          <span class="footer-msg">All conflicts resolved.</span>
          <button class="save-btn" onclick={saveAndStage} disabled={isSaving}>
            {isSaving ? 'Saving…' : 'Save & Stage'}
          </button>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .conflict-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .conflict-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    gap: 8px;
  }

  .conflict-path {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .badge {
    border-radius: 10px;
    padding: 1px 8px;
    font-size: 11px;
  }

  .badge-danger {
    background: color-mix(in srgb, var(--danger) 20%, transparent);
    color: var(--danger);
  }

  .badge-success {
    background: color-mix(in srgb, var(--success) 20%, transparent);
    color: var(--success);
  }

  .status-badge {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .action-btn {
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: none;
    color: var(--text-secondary);
    font-size: 11px;
    cursor: pointer;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--bg-surface);
    color: var(--text-primary);
  }

  .action-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .conflict-body {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
  }

  .conflict-content {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 12px;
    flex: 1;
  }

  .message {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }

  .message.error {
    color: var(--danger);
  }

  /* Diff lines */
  .diff-line {
    display: flex;
    align-items: baseline;
    line-height: 20px;
    white-space: pre;
  }

  .line-context {
    color: var(--text-primary);
  }

  .line-ours {
    background: color-mix(in srgb, var(--success) 15%, transparent);
    color: var(--success);
  }

  .line-theirs {
    background: color-mix(in srgb, #7c6fcd 15%, transparent);
    color: #a79fdf;
  }

  .line-ours.dimmed,
  .line-theirs.dimmed {
    opacity: 0.3;
  }

  .line-no {
    width: 45px;
    text-align: right;
    padding-right: 8px;
    color: var(--text-secondary);
    opacity: 0.6;
    user-select: none;
    flex-shrink: 0;
  }

  .line-prefix {
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    user-select: none;
  }

  .line-content {
    flex: 1;
    padding-right: 12px;
  }

  /* Hunk headers */
  .hunk-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    font-size: 11px;
    font-family: monospace;
  }

  .hunk-header-ours {
    background: color-mix(in srgb, var(--success) 8%, var(--bg-secondary));
    color: var(--success);
  }

  .hunk-header-theirs {
    background: color-mix(in srgb, #7c6fcd 8%, var(--bg-secondary));
    color: #a79fdf;
  }

  .hunk-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .hunk-actions {
    flex-shrink: 0;
    margin-left: 8px;
  }

  .hunk-separator {
    padding: 2px 12px;
    background: var(--bg-secondary);
    color: var(--text-secondary);
    font-size: 11px;
    font-family: monospace;
    border-top: 1px dashed var(--border);
    border-bottom: 1px dashed var(--border);
  }

  .hunk-end {
    padding: 2px 12px;
    background: color-mix(in srgb, #7c6fcd 8%, var(--bg-secondary));
    color: #a79fdf;
    font-size: 11px;
    font-family: monospace;
    border-bottom: 1px solid var(--border);
  }

  /* Accept buttons */
  .accept-btn {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .accept-ours {
    border: 1px solid var(--success);
    background: color-mix(in srgb, var(--success) 15%, transparent);
    color: var(--success);
  }

  .accept-theirs {
    border: 1px solid #7c6fcd;
    background: color-mix(in srgb, #7c6fcd 15%, transparent);
    color: #a79fdf;
  }

  .resolved-badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 10px;
  }

  .badge-theirs {
    background: color-mix(in srgb, #7c6fcd 20%, transparent);
    color: #a79fdf;
  }

  /* Footer */
  .conflict-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    padding: 8px 12px;
    border-top: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .footer-msg {
    font-size: 12px;
    color: var(--success);
  }

  .save-btn {
    padding: 4px 14px;
    border: none;
    border-radius: 4px;
    background: var(--accent);
    color: var(--bg-primary);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .save-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
