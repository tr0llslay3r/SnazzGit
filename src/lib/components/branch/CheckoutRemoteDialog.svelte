<script lang="ts">
  import { showCheckoutRemoteDialog, checkoutRemoteBranch, addToast } from '$lib/stores/ui';
  import { repoInfo, refreshAll } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';

  let localName = $state('');
  let track = $state(true);
  let inputEl: HTMLInputElement = $state(null!);

  $effect(() => {
    if ($showCheckoutRemoteDialog && $checkoutRemoteBranch) {
      const parts = $checkoutRemoteBranch.name.split('/');
      localName = parts.slice(1).join('/');
      track = true;
    }
  });

  $effect(() => {
    if ($showCheckoutRemoteDialog && inputEl) {
      inputEl.focus();
      inputEl.select();
    }
  });

  async function checkout() {
    if (!$repoInfo || !$checkoutRemoteBranch || !localName.trim()) return;
    try {
      await tauri.checkoutRemoteBranch(
        $repoInfo.path,
        $checkoutRemoteBranch.name,
        localName.trim(),
        track,
      );
      await refreshAll();
      addToast(`Checked out ${localName.trim()} tracking ${$checkoutRemoteBranch.name}`, 'success');
      close();
    } catch (e) {
      addToast(`Checkout failed: ${e}`, 'error');
    }
  }

  function close() {
    localName = '';
    $showCheckoutRemoteDialog = false;
    $checkoutRemoteBranch = null;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') checkout();
    if (e.key === 'Escape') close();
  }
</script>

{#if $showCheckoutRemoteDialog && $checkoutRemoteBranch}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="dialog-overlay" onclick={close}>
    <!-- svelte-ignore a11y_interactive_supports_focus, a11y_no_static_element_interactions -->
    <div class="dialog" onclick={(e) => e.stopPropagation()} onkeydown={onKeydown} role="dialog" tabindex="-1">
      <h3 class="dialog-title">Checkout Remote Branch</h3>
      <div class="remote-name">{$checkoutRemoteBranch.name}</div>
      <label class="field-label" for="local-branch-name">Local branch name</label>
      <input
        class="dialog-input"
        type="text"
        id="local-branch-name"
        bind:this={inputEl}
        bind:value={localName}
      />
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={track} />
        Track remote branch
      </label>
      <div class="dialog-actions">
        <button class="btn-secondary" onclick={close}>Cancel</button>
        <button class="btn-primary" onclick={checkout} disabled={!localName.trim()}>Checkout</button>
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
    margin: 0 0 12px;
    font-size: 16px;
    color: var(--text-primary);
  }
  .remote-name {
    font-size: 13px;
    color: var(--text-secondary);
    background: var(--bg-primary);
    padding: 6px 10px;
    border-radius: 4px;
    margin-bottom: 12px;
    font-family: monospace;
  }
  .field-label {
    display: block;
    font-size: 12px;
    color: var(--text-secondary);
    margin-bottom: 4px;
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
  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 12px;
    font-size: 13px;
    color: var(--text-primary);
    cursor: pointer;
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
