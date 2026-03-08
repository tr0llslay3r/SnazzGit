<script lang="ts">
  import { showMergeDialog, addToast } from '$lib/stores/ui';
  import { repoInfo, localBranches, refreshAll } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';

  let selectedBranch = $state('');

  let mergeBranches = $derived($localBranches.filter((b) => !b.is_head));

  async function doMerge() {
    if (!$repoInfo || !selectedBranch) return;
    try {
      const result = await tauri.mergeBranch($repoInfo.path, selectedBranch);
      await refreshAll();
      addToast(`Merge: ${result}`, 'success');
      $showMergeDialog = false;
    } catch (e) {
      addToast(`Merge failed: ${e}`, 'error');
    }
  }
</script>

{#if $showMergeDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="dialog-overlay" onclick={() => $showMergeDialog = false}>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="dialog" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') $showMergeDialog = false; }} role="dialog" tabindex="-1">
      <h3 class="dialog-title">Merge Branch</h3>
      <p class="dialog-desc">Merge into current branch ({$repoInfo?.current_branch})</p>
      <select class="dialog-select" bind:value={selectedBranch}>
        <option value="">Select branch...</option>
        {#each mergeBranches as branch}
          <option value={branch.name}>{branch.name}</option>
        {/each}
      </select>
      <div class="dialog-actions">
        <button class="btn-secondary" onclick={() => $showMergeDialog = false}>Cancel</button>
        <button class="btn-primary" onclick={doMerge} disabled={!selectedBranch}>Merge</button>
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
    width: 400px;
    max-width: 90vw;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }
  .dialog-title {
    margin: 0 0 4px;
    font-size: 16px;
    color: var(--text-primary);
  }
  .dialog-desc {
    margin: 0 0 16px;
    font-size: 13px;
    color: var(--text-secondary);
  }
  .dialog-select {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 14px;
    outline: none;
  }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
  .btn-secondary {
    padding: 6px 16px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
  }
  .btn-primary {
    padding: 6px 16px;
    border: none;
    border-radius: 6px;
    background: var(--accent);
    color: var(--bg-primary);
    cursor: pointer;
    font-weight: 600;
  }
  .btn-primary:disabled { opacity: 0.4; }
</style>
