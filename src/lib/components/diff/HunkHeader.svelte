<script lang="ts">
  import type { DiffHunk } from '$lib/types';

  interface Props {
    hunk: DiffHunk;
    onStage?: () => void;
    onDiscard?: () => void;
    showActions?: boolean;
  }

  let { hunk, onStage, onDiscard, showActions = false }: Props = $props();
</script>

<div class="hunk-header">
  <span class="hunk-info">{hunk.header}</span>
  {#if showActions}
    <div class="hunk-actions">
      {#if onStage}
        <button class="hunk-btn stage" onclick={onStage} title="Stage hunk">Stage</button>
      {/if}
      {#if onDiscard}
        <button class="hunk-btn discard" onclick={onDiscard} title="Discard hunk">Discard</button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .hunk-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    background: color-mix(in srgb, var(--accent) 10%, var(--bg-secondary));
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    font-family: monospace;
    font-size: 12px;
    color: var(--text-secondary);
  }
  .hunk-actions {
    display: flex;
    gap: 6px;
  }
  .hunk-btn {
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    background: var(--bg-surface);
    color: var(--text-secondary);
  }
  .hunk-btn:hover {
    color: var(--text-primary);
  }
  .hunk-btn.discard:hover {
    border-color: var(--danger);
    color: var(--danger);
  }
  .hunk-btn.stage:hover {
    border-color: var(--success);
    color: var(--success);
  }
</style>
