<script lang="ts">
  import { interactiveRebaseStatus, repoInfo, refreshAll, refreshInteractiveRebaseStatus } from '$lib/stores/repo';
  import { addToast } from '$lib/stores/ui';
  import * as tauri from '$lib/utils/tauri';

  let busy = $state(false);

  async function continueRebase() {
    if (!$repoInfo || busy) return;
    busy = true;
    try {
      const result = await tauri.interactiveRebaseContinue($repoInfo.path);
      await refreshAll();
      await refreshInteractiveRebaseStatus();
      addToast(result, result.toLowerCase().includes('paused') ? 'warning' : 'success');
    } catch (e) {
      addToast(`Continue failed: ${e}`, 'error');
    } finally {
      busy = false;
    }
  }

  async function abortRebase() {
    if (!$repoInfo || busy) return;
    busy = true;
    try {
      await tauri.interactiveRebaseAbort($repoInfo.path);
      await refreshAll();
      await refreshInteractiveRebaseStatus();
      addToast('Interactive rebase aborted', 'success');
    } catch (e) {
      addToast(`Abort failed: ${e}`, 'error');
    } finally {
      busy = false;
    }
  }

  function reasonLabel(reason: string | null): string {
    if (reason === 'conflict') return 'conflicts';
    if (reason === 'edit') return 'editing';
    return '';
  }
</script>

{#if $interactiveRebaseStatus?.active}
  <div class="rebase-banner">
    <span class="rebase-text">
      Interactive rebase: step {$interactiveRebaseStatus.completed_steps + 1}/{$interactiveRebaseStatus.total_steps}
      — paused ({reasonLabel($interactiveRebaseStatus.paused_reason)})
    </span>
    <div class="rebase-actions">
      <button class="btn-continue" onclick={continueRebase} disabled={busy}>Continue</button>
      <button class="btn-abort" onclick={abortRebase} disabled={busy}>Abort</button>
    </div>
  </div>
{/if}

<style>
  .rebase-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 16px;
    background: color-mix(in srgb, var(--warning) 15%, var(--bg-secondary));
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    flex-shrink: 0;
  }
  .rebase-text {
    color: var(--text-primary);
  }
  .rebase-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }
  .btn-continue,
  .btn-abort {
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: 12px;
    cursor: pointer;
  }
  .btn-continue:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
  }
  .btn-abort:hover:not(:disabled) {
    border-color: var(--danger);
    color: var(--danger);
  }
  .btn-continue:disabled,
  .btn-abort:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
