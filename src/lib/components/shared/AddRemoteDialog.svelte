<script lang="ts">
  import { showAddRemoteDialog, addToast } from '$lib/stores/ui';
  import { repoInfo, refreshAll } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';

  let remoteName = $state('');
  let remoteUrl = $state('');
  let nameInput: HTMLInputElement | undefined = $state();

  $effect(() => {
    if ($showAddRemoteDialog && nameInput) {
      nameInput.focus();
    }
  });

  async function submit() {
    if (!remoteName.trim() || !remoteUrl.trim() || !$repoInfo) return;
    try {
      await tauri.addRemote($repoInfo.path, remoteName.trim(), remoteUrl.trim());
      await refreshAll();
      addToast(`Added remote "${remoteName.trim()}"`, 'success');
      close();
    } catch (e) {
      addToast(`Add remote failed: ${e}`, 'error');
    }
  }

  function close() {
    $showAddRemoteDialog = false;
    remoteName = '';
    remoteUrl = '';
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      submit();
    }
    if (e.key === 'Escape') {
      close();
    }
  }
</script>

{#if $showAddRemoteDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="dialog-overlay" onclick={close}>
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="dialog" onclick={(e) => e.stopPropagation()} onkeydown={onKeydown}>
      <h3>Add Remote</h3>
      <label class="field-label">
        Name
        <input
          type="text"
          bind:this={nameInput}
          bind:value={remoteName}
          placeholder="e.g. origin, upstream"
        />
      </label>
      <label class="field-label">
        URL
        <input
          type="text"
          bind:value={remoteUrl}
          placeholder="https://github.com/user/repo.git"
        />
      </label>
      <div class="dialog-actions">
        <button class="btn-cancel" onclick={close}>Cancel</button>
        <button class="btn-primary" onclick={submit} disabled={!remoteName.trim() || !remoteUrl.trim()}>Add Remote</button>
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
    min-width: 380px;
    max-width: 480px;
  }
  h3 {
    margin: 0 0 16px;
    font-size: 16px;
    color: var(--text-primary);
  }
  .field-label {
    display: block;
    font-size: 12px;
    color: var(--text-secondary);
    margin-bottom: 12px;
  }
  .field-label input {
    display: block;
    width: 100%;
    margin-top: 4px;
    padding: 8px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
    box-sizing: border-box;
  }
  .field-label input:focus {
    border-color: var(--accent);
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
