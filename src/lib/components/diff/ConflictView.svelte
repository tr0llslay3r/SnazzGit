<script lang="ts">
  import { repoInfo, refreshStatus } from '$lib/stores/repo';
  import { addToast } from '$lib/stores/ui';
  import * as tauri from '$lib/utils/tauri';
  import type { ConflictFile, ConflictHunk } from '$lib/types';

  interface Props {
    filePath: string;
  }

  let { filePath }: Props = $props();

  // ── State ──────────────────────────────────────────────────────────────
  let conflictFile = $state<ConflictFile | null>(null);
  let loadError = $state<string | null>(null);
  let isLoading = $state(false);
  let choices = $state(new Map<number, 'ours' | 'theirs'>());
  let resultContent = $state('');
  let isSaving = $state(false);

  // ── Types ──────────────────────────────────────────────────────────────
  type ContextSeg = { type: 'context'; lines: string[]; startLine: number };
  type ConflictSeg = {
    type: 'conflict';
    idx: number;
    oursLines: string[];
    oursStart: number;
    theirsLines: string[];
    theirsStart: number;
  };
  type Segment = ContextSeg | ConflictSeg;

  // ── Helpers ────────────────────────────────────────────────────────────
  function buildSegments(cf: ConflictFile | null): Segment[] {
    if (!cf) return [];
    const segs: Segment[] = [];
    const theirsMap = new Map<number, ConflictHunk>();

    for (const h of cf.hunks) {
      if (h.kind === 'theirs' && h.conflict_index !== null) {
        theirsMap.set(h.conflict_index, h);
      }
    }

    for (const h of cf.hunks) {
      if (h.kind === 'context') {
        segs.push({ type: 'context', lines: h.lines, startLine: h.start_line });
      } else if (h.kind === 'ours' && h.conflict_index !== null) {
        const theirs = theirsMap.get(h.conflict_index);
        if (theirs) {
          segs.push({
            type: 'conflict',
            idx: h.conflict_index,
            oursLines: h.lines,
            oursStart: h.start_line,
            theirsLines: theirs.lines,
            theirsStart: theirs.start_line,
          });
        }
      }
    }
    return segs;
  }

  function buildResult(segs: Segment[], ch: Map<number, 'ours' | 'theirs'>): string {
    const parts: string[] = [];
    for (const seg of segs) {
      if (seg.type === 'context') {
        parts.push(...seg.lines);
      } else {
        const choice = ch.get(seg.idx);
        if (choice === 'ours') {
          parts.push(...seg.oursLines);
        } else if (choice === 'theirs') {
          parts.push(...seg.theirsLines);
        } else {
          parts.push(`<<<<<<< CONFLICT ${seg.idx + 1} — unresolved >>>>>>>\n`);
        }
      }
    }
    return parts.join('');
  }

  // ── Derived ────────────────────────────────────────────────────────────
  let segments = $derived(buildSegments(conflictFile));
  let conflictCount = $derived(segments.filter((s) => s.type === 'conflict').length);
  let resolvedCount = $derived(choices.size);
  let unresolvedCount = $derived(conflictCount - resolvedCount);
  let hasPlaceholders = $derived(resultContent.includes('<<<<<<< CONFLICT'));

  // ── Load ───────────────────────────────────────────────────────────────
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
    resultContent = '';
    try {
      const cf = await tauri.getConflictDiff($repoInfo!.path, filePath);
      conflictFile = cf;
      resultContent = buildResult(buildSegments(cf), new Map());
    } catch (err) {
      loadError = String(err);
    } finally {
      isLoading = false;
    }
  }

  // ── Per-conflict actions ───────────────────────────────────────────────
  function acceptOurs(idx: number) {
    const next = new Map(choices).set(idx, 'ours');
    choices = next;
    resultContent = buildResult(segments, next);
  }

  function acceptTheirs(idx: number) {
    const next = new Map(choices).set(idx, 'theirs');
    choices = next;
    resultContent = buildResult(segments, next);
  }

  // ── Global actions (use git index stage blobs) ─────────────────────────
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

  // ── Save (uses current textarea content verbatim) ──────────────────────
  async function saveAndStage() {
    if (!$repoInfo) return;
    isSaving = true;
    try {
      await tauri.saveResolvedConflict($repoInfo.path, filePath, resultContent);
      await refreshStatus();
      addToast('Conflict resolved and staged', 'success');
    } catch (e) {
      addToast(`Failed to save: ${e}`, 'error');
    } finally {
      isSaving = false;
    }
  }
</script>

<div class="conflict-view">
  <!-- ── Toolbar ──────────────────────────────────────────────────────── -->
  <div class="toolbar">
    <span class="path">{filePath}</span>
    <div class="toolbar-right">
      {#if !isLoading && conflictFile}
        {#if unresolvedCount > 0}
          <span class="badge badge-warn">{unresolvedCount} unresolved</span>
        {:else}
          <span class="badge badge-ok">All resolved</span>
        {/if}
      {/if}
      <button class="btn" onclick={acceptAllOurs} disabled={isLoading || !conflictFile}>
        Accept All Ours
      </button>
      <button class="btn" onclick={acceptAllTheirs} disabled={isLoading || !conflictFile}>
        Accept All Theirs
      </button>
    </div>
  </div>

  <!-- ── Loading / error ─────────────────────────────────────────────── -->
  {#if isLoading}
    <div class="message">Loading…</div>
  {:else if loadError}
    <div class="message error">{loadError}</div>
  {:else if conflictFile}
    <!-- ── Body ──────────────────────────────────────────────────────── -->
    <div class="body">

      <!-- ── Top: side-by-side ─────────────────────────────────────── -->
      <div class="side-by-side">
        <!-- Sticky column headers -->
        <div class="panel-labels">
          <div class="panel-label label-ours">Ours — {conflictFile.our_label}</div>
          <div class="panel-label label-theirs">Theirs — {conflictFile.their_label}</div>
        </div>

        <!-- Scrollable two-column grid (auto-aligns rows) -->
        <div class="merge-scroll">
          <div class="merge-grid">
            {#each segments as seg}
              {#if seg.type === 'context'}
                <!-- Both columns show identical context -->
                <div class="cell cell-ctx cell-br">
                  {#each seg.lines as line, i}
                    <div class="code-row">
                      <span class="lnum">{seg.startLine + i}</span>
                      <span class="code">{line}</span>
                    </div>
                  {/each}
                </div>
                <div class="cell cell-ctx">
                  {#each seg.lines as line, i}
                    <div class="code-row">
                      <span class="lnum">{seg.startLine + i}</span>
                      <span class="code">{line}</span>
                    </div>
                  {/each}
                </div>

              {:else if seg.type === 'conflict'}
                {@const choice = choices.get(seg.idx)}

                <!-- Ours column -->
                <div class="cell cell-ours cell-br" class:chosen={choice === 'ours'}>
                  <div class="cell-header cell-header-ours">
                    <span class="cnum">#{seg.idx + 1}</span>
                    <button
                      class="use-btn use-ours"
                      class:active={choice === 'ours'}
                      onclick={() => acceptOurs(seg.idx)}
                    >
                      {choice === 'ours' ? '✓ Using Ours' : 'Use Ours ↓'}
                    </button>
                  </div>
                  {#each seg.oursLines as line, i}
                    <div class="code-row row-ours">
                      <span class="lnum">{seg.oursStart + i}</span>
                      <span class="code">{line}</span>
                    </div>
                  {/each}
                </div>

                <!-- Theirs column -->
                <div class="cell cell-theirs" class:chosen={choice === 'theirs'}>
                  <div class="cell-header cell-header-theirs">
                    <span class="cnum">#{seg.idx + 1}</span>
                    <button
                      class="use-btn use-theirs"
                      class:active={choice === 'theirs'}
                      onclick={() => acceptTheirs(seg.idx)}
                    >
                      {choice === 'theirs' ? '✓ Using Theirs' : 'Use Theirs ↓'}
                    </button>
                  </div>
                  {#each seg.theirsLines as line, i}
                    <div class="code-row row-theirs">
                      <span class="lnum">{seg.theirsStart + i}</span>
                      <span class="code">{line}</span>
                    </div>
                  {/each}
                </div>
              {/if}
            {/each}
          </div>
        </div>
      </div>

      <!-- ── Splitter ───────────────────────────────────────────────── -->
      <div class="splitter"></div>

      <!-- ── Bottom: editable result ───────────────────────────────── -->
      <div class="result-panel">
        <div class="result-header">
          Result
          <span class="result-hint">— editable, saved as-is on Save &amp; Stage</span>
        </div>
        <textarea
          class="result-textarea"
          bind:value={resultContent}
          spellcheck="false"
          autocomplete="off"
          autocapitalize="off"
        ></textarea>
      </div>
    </div>

    <!-- ── Footer ──────────────────────────────────────────────────────── -->
    <div class="footer">
      {#if hasPlaceholders}
        <span class="footer-warn">⚠ Unresolved conflicts remain in result</span>
      {/if}
      <button class="save-btn" onclick={saveAndStage} disabled={isSaving}>
        {isSaving ? 'Saving…' : 'Save & Stage'}
      </button>
    </div>
  {/if}
</div>

<style>
  /* ── Shell ────────────────────────────────────────────────────────────── */
  .conflict-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    font-family: 'JetBrains Mono', 'Fira Code', ui-monospace, monospace;
    font-size: 12px;
  }

  /* ── Toolbar ──────────────────────────────────────────────────────────── */
  .toolbar {
    display: flex;
    align-items: center;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    gap: 8px;
  }

  .path {
    flex: 1;
    color: var(--text-primary);
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
    font-family: ui-sans-serif, system-ui, sans-serif;
  }
  .badge-warn { background: color-mix(in srgb, var(--danger) 15%, transparent); color: var(--danger); }
  .badge-ok   { background: color-mix(in srgb, var(--success) 15%, transparent); color: var(--success); }

  .btn {
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: none;
    color: var(--text-secondary);
    font-size: 11px;
    font-family: ui-sans-serif, system-ui, sans-serif;
    cursor: pointer;
  }
  .btn:hover:not(:disabled) { background: var(--bg-surface); color: var(--text-primary); }
  .btn:disabled { opacity: 0.4; cursor: default; }

  /* ── Loading / error ──────────────────────────────────────────────────── */
  .message {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    font-family: ui-sans-serif, system-ui, sans-serif;
  }
  .message.error { color: var(--danger); }

  /* ── Body: column flex containing side-by-side + result ─────────────── */
  .body {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  /* ── Side-by-side (top 60%) ───────────────────────────────────────────── */
  .side-by-side {
    flex: 3;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  /* Fixed column label bar */
  .panel-labels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    flex-shrink: 0;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
  }

  .panel-label {
    padding: 5px 12px;
    font-size: 11px;
    font-weight: 600;
    font-family: ui-sans-serif, system-ui, sans-serif;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .label-ours   { color: var(--success); border-right: 1px solid var(--border); }
  .label-theirs { color: #a79fdf; }

  /* Single scrollable area — one scroll bar governs both columns */
  .merge-scroll {
    flex: 1;
    overflow: auto;
  }

  /* CSS grid auto-aligns ours/theirs cells to the same row height */
  .merge-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  /* ── Grid cells ───────────────────────────────────────────────────────── */
  .cell {
    border-bottom: 1px solid var(--border);
    min-width: 0; /* prevent grid blowout */
    overflow: hidden;
  }

  .cell-br { border-right: 1px solid var(--border); }

  .cell-ctx { background: var(--bg-primary); }

  .cell-ours   { background: color-mix(in srgb, var(--success) 4%, var(--bg-primary)); }
  .cell-theirs { background: color-mix(in srgb, #7c6fcd 4%, var(--bg-primary)); }

  .cell-ours.chosen   { background: color-mix(in srgb, var(--success) 11%, var(--bg-primary)); }
  .cell-theirs.chosen { background: color-mix(in srgb, #7c6fcd 11%, var(--bg-primary)); }

  /* ── Conflict cell header row ─────────────────────────────────────────── */
  .cell-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 3px 8px;
    border-bottom: 1px dashed var(--border);
    background: var(--bg-secondary);
  }

  .cell-header-ours   { border-left: 2px solid var(--success); }
  .cell-header-theirs { border-left: 2px solid #7c6fcd; }

  .cnum {
    font-size: 10px;
    color: var(--text-secondary);
    font-family: ui-sans-serif, system-ui, sans-serif;
  }

  /* ── Code rows ────────────────────────────────────────────────────────── */
  .code-row {
    display: flex;
    align-items: baseline;
    height: 20px;
    white-space: pre;
    overflow: hidden;
  }

  .lnum {
    width: 40px;
    flex-shrink: 0;
    text-align: right;
    padding-right: 6px;
    color: var(--text-secondary);
    opacity: 0.5;
    user-select: none;
    font-size: 11px;
  }

  .code {
    flex: 1;
    padding: 0 6px;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-primary);
  }

  .row-ours   .code { color: var(--success); }
  .row-theirs .code { color: #a79fdf; }

  /* ── Use buttons ──────────────────────────────────────────────────────── */
  .use-btn {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-family: ui-sans-serif, system-ui, sans-serif;
    cursor: pointer;
    border: 1px solid;
  }

  .use-ours  { border-color: var(--success); background: color-mix(in srgb, var(--success) 15%, transparent); color: var(--success); }
  .use-theirs { border-color: #7c6fcd; background: color-mix(in srgb, #7c6fcd 15%, transparent); color: #a79fdf; }

  .use-ours.active   { background: var(--success); color: var(--bg-primary); }
  .use-theirs.active { background: #7c6fcd; color: #fff; }

  /* ── Splitter between top and bottom panels ───────────────────────────── */
  .splitter {
    height: 4px;
    background: var(--border);
    flex-shrink: 0;
  }

  /* ── Result panel (bottom 40%) ────────────────────────────────────────── */
  .result-panel {
    flex: 2;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  .result-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 12px;
    font-size: 11px;
    font-weight: 600;
    font-family: ui-sans-serif, system-ui, sans-serif;
    color: var(--text-secondary);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .result-hint {
    font-weight: 400;
    font-style: italic;
    opacity: 0.7;
  }

  .result-textarea {
    flex: 1;
    width: 100%;
    box-sizing: border-box;
    background: var(--bg-primary);
    color: var(--text-primary);
    border: none;
    outline: none;
    resize: none;
    font-family: 'JetBrains Mono', 'Fira Code', ui-monospace, monospace;
    font-size: 12px;
    line-height: 20px;
    padding: 8px 12px;
    tab-size: 4;
  }

  /* ── Footer ───────────────────────────────────────────────────────────── */
  .footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    padding: 8px 12px;
    border-top: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .footer-warn {
    font-size: 11px;
    color: var(--warning, #f59e0b);
    font-family: ui-sans-serif, system-ui, sans-serif;
  }

  .save-btn {
    padding: 4px 16px;
    border: none;
    border-radius: 4px;
    background: var(--accent);
    color: var(--bg-primary);
    font-size: 12px;
    font-weight: 600;
    font-family: ui-sans-serif, system-ui, sans-serif;
    cursor: pointer;
  }
  .save-btn:disabled { opacity: 0.4; cursor: default; }
</style>
