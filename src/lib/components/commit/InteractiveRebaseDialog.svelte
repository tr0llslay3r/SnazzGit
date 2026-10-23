<script lang="ts">
  import {
    showInteractiveRebaseDialog,
    interactiveRebaseBaseCommitId,
    interactiveRebaseInitialTodo,
    addToast,
  } from '$lib/stores/ui';
  import { repoInfo, refreshAll, refreshInteractiveRebaseStatus } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';
  import type { InteractiveRebaseRow, RebaseAction } from '$lib/types';

  let rows = $state<InteractiveRebaseRow[]>([]);
  let submitting = $state(false);

  $effect(() => {
    if ($showInteractiveRebaseDialog) {
      rows = $interactiveRebaseInitialTodo.map((r) => ({ ...r, message: r.message ?? '' }));
    }
  });

  const ACTIONS: { value: RebaseAction; label: string }[] = [
    { value: 'pick', label: 'Pick' },
    { value: 'reword', label: 'Reword' },
    { value: 'edit', label: 'Edit' },
    { value: 'squash', label: 'Squash' },
    { value: 'fixup', label: 'Fixup' },
    { value: 'drop', label: 'Drop' },
  ];

  function actionsFor(index: number) {
    // The first item has nothing before it to fold into.
    return index === 0 ? ACTIONS.filter((a) => a.value !== 'squash' && a.value !== 'fixup') : ACTIONS;
  }

  function needsMessage(action: RebaseAction): boolean {
    return action === 'reword' || action === 'squash';
  }

  function moveUp(index: number) {
    if (index <= 0) return;
    const copy = rows.slice();
    [copy[index - 1], copy[index]] = [copy[index], copy[index - 1]];
    rows = copy;
  }

  function moveDown(index: number) {
    if (index >= rows.length - 1) return;
    const copy = rows.slice();
    [copy[index], copy[index + 1]] = [copy[index + 1], copy[index]];
    rows = copy;
  }

  function close() {
    $showInteractiveRebaseDialog = false;
    $interactiveRebaseBaseCommitId = null;
    rows = [];
  }

  async function confirm() {
    if (!$repoInfo || !$interactiveRebaseBaseCommitId || rows.length === 0 || submitting) return;
    submitting = true;
    try {
      const todo = rows.map((r) => ({
        commit_id: r.commit.id,
        action: r.action,
        message: needsMessage(r.action) && r.message?.trim() ? r.message.trim() : null,
      }));
      const result = await tauri.startInteractiveRebase(
        $repoInfo.path,
        $interactiveRebaseBaseCommitId,
        todo,
      );
      await refreshAll();
      await refreshInteractiveRebaseStatus();
      addToast(result, result.toLowerCase().includes('paused') ? 'warning' : 'success');
      close();
    } catch (e) {
      addToast(`Interactive rebase failed: ${e}`, 'error');
    } finally {
      submitting = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

{#if $showInteractiveRebaseDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="dialog-overlay" onclick={close}>
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="dialog" onclick={(e) => e.stopPropagation()} onkeydown={onKeydown}>
      <h3>Interactive Rebase</h3>
      <p class="hint">Reorder, reword, squash, fixup, or drop commits before replaying them.</p>

      <div class="row-list">
        {#if rows.length === 0}
          <div class="empty-state">Nothing to rebase</div>
        {:else}
          {#each rows as row, i (row.commit.id)}
            <div class="todo-row">
              <div class="reorder">
                <button class="reorder-btn" disabled={i === 0} onclick={() => moveUp(i)} title="Move up">&uarr;</button>
                <button class="reorder-btn" disabled={i === rows.length - 1} onclick={() => moveDown(i)} title="Move down">&darr;</button>
              </div>
              <select class="action-select" bind:value={row.action}>
                {#each actionsFor(i) as opt (opt.value)}
                  <option value={opt.value}>{opt.label}</option>
                {/each}
              </select>
              <span class="commit-hash">{row.commit.short_id}</span>
              <span class="commit-summary" class:dropped={row.action === 'drop'}>{row.commit.summary}</span>
            </div>
            {#if needsMessage(row.action)}
              <textarea
                class="message-input"
                rows="2"
                placeholder={row.action === 'squash'
                  ? 'Combined message (optional — defaults to both messages)'
                  : 'New commit message (optional — defaults to original)'}
                bind:value={row.message}
              ></textarea>
            {/if}
          {/each}
        {/if}
      </div>

      <div class="dialog-actions">
        <button class="btn-cancel" onclick={close}>Cancel</button>
        <button class="btn-primary" onclick={confirm} disabled={rows.length === 0 || submitting}>
          {submitting ? 'Working…' : 'Start Rebase'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    width: 600px;
    max-width: 90vw;
  }
  h3 {
    margin: 0 0 4px;
    font-size: 16px;
    color: var(--text-primary);
  }
  .hint {
    margin: 0 0 14px;
    font-size: 12px;
    color: var(--text-secondary);
  }
  .row-list {
    max-height: 400px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px;
  }
  .empty-state {
    padding: 16px;
    text-align: center;
    color: var(--text-secondary);
    font-size: 13px;
  }
  .todo-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
  }
  .todo-row:hover {
    background: var(--bg-surface);
    border-radius: 6px;
  }
  .reorder {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex-shrink: 0;
  }
  .reorder-btn {
    border: none;
    background: var(--bg-primary);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 10px;
    line-height: 1;
    padding: 1px 4px;
    border-radius: 3px;
  }
  .reorder-btn:hover:not(:disabled) {
    color: var(--accent);
  }
  .reorder-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .action-select {
    flex-shrink: 0;
    padding: 4px 6px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 12px;
  }
  .commit-hash {
    font-family: monospace;
    color: var(--accent);
    flex-shrink: 0;
  }
  .commit-summary {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: var(--text-primary);
  }
  .commit-summary.dropped {
    text-decoration: line-through;
    color: var(--text-secondary);
  }
  .message-input {
    width: 100%;
    margin: 2px 0 6px;
    padding: 6px 8px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: inherit;
    resize: vertical;
    box-sizing: border-box;
  }
  .message-input:focus {
    border-color: var(--accent);
    outline: none;
  }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
  .btn-cancel {
    padding: 6px 16px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 13px;
  }
  .btn-primary {
    padding: 6px 16px;
    border: none;
    border-radius: 6px;
    background: var(--accent);
    color: var(--bg-primary);
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
  }
  .btn-primary:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .btn-primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }
</style>
