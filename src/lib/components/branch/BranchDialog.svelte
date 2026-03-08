<script lang="ts">
  import { showBranchDialog, addToast } from '$lib/stores/ui';
  import { repoInfo, refreshAll } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';

  let branchName = $state('');
  let inputEl: HTMLInputElement = $state(null!);

  $effect(() => {
    if ($showBranchDialog && inputEl) {
      inputEl.focus();
    }
  });

  async function create() {
    if (!$repoInfo || !branchName.trim()) return;
    try {
      await tauri.createBranch($repoInfo.path, branchName.trim());
      await refreshAll();
      addToast(`Branch '${branchName}' created`, 'success');
      branchName = '';
      $showBranchDialog = false;
    } catch (e) {
      addToast(`Failed to create branch: ${e}`, 'error');
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') create();
    if (e.key === 'Escape') { branchName = ''; $showBranchDialog = false; }
  }
</script>

{#if $showBranchDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="dialog-overlay" onclick={() => { branchName = ''; $showBranchDialog = false; }}>
    <!-- svelte-ignore a11y_interactive_supports_focus, a11y_no_static_element_interactions -->
    <div class="dialog" onclick={(e) => e.stopPropagation()} onkeydown={onKeydown} role="dialog" tabindex="-1">
      <h3 class="dialog-title">New Branch</h3>
      <input
        class="dialog-input"
        type="text"
        placeholder="Branch name..."
        bind:this={inputEl}
        bind:value={branchName}
      />
      <div class="dialog-actions">
        <button class="btn-secondary" onclick={() => { branchName = ''; $showBranchDialog = false; }}>Cancel</button>
        <button class="btn-primary" onclick={create} disabled={!branchName.trim()}>Create</button>
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
    margin: 0 0 16px;
    font-size: 16px;
    color: var(--text-primary);
  }
  .dialog-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 14px;
    outline: none;
    box-sizing: border-box;
  }
  .dialog-input:focus {
    border-color: var(--accent);
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
  }
</style>
